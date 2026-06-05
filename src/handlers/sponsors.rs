use actix_web::{delete, get, post, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::errors::AppError;
use crate::handlers::auth::current_user;
use crate::models::portfolio::{Fund, Sponsor};
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// A sponsor decorated with aggregated portfolio metrics.
#[derive(Debug, Serialize)]
pub struct SponsorWithMetrics {
    #[serde(flatten)]
    pub sponsor: Sponsor,
    pub fund_count: usize,
    pub total_nav: f64,
    pub total_commitment: f64,
    pub tvpi: f64,
    pub net_irr: f64,
    pub asset_classes: Vec<String>,
    pub company_count: usize,
}

// ---------------------------------------------------------------------------
// Query parameter & body structs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SponsorFilter {
    pub entity: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FundFilter {
    pub entity: Option<String>,
    pub sponsor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSponsorPayload {
    pub name: String,
    pub initials: String,
    pub country: String,
    pub color: String,
}

/// PostgREST insert payload for a sponsor — server controls id + org.
#[derive(Serialize)]
struct NewSponsor<'a> {
    id: &'a str,
    organization_id: &'a str,
    name: &'a str,
    initials: &'a str,
    country: &'a str,
    color: &'a str,
}

// ---------------------------------------------------------------------------
// Aggregation helper
// ---------------------------------------------------------------------------

fn aggregate(sponsor: &Sponsor, funds: &[&Fund]) -> SponsorWithMetrics {
    let fund_count = funds.len();
    let total_nav: f64 = funds.iter().map(|f| f.nav).sum();
    let total_commitment: f64 = funds.iter().map(|f| f.commitment).sum();
    let company_count: usize = funds.iter().map(|f| f.companies.len()).sum();

    let total_paid_in: f64 = funds.iter().map(|f| f.paid_in).sum();
    let (tvpi, net_irr) = if total_paid_in > 0.0 {
        let w_tvpi: f64 = funds.iter().map(|f| f.tvpi * f.paid_in).sum();
        let w_irr: f64 = funds.iter().map(|f| f.net_irr * f.paid_in).sum();
        (w_tvpi / total_paid_in, w_irr / total_paid_in)
    } else {
        (0.0, 0.0)
    };

    let mut asset_classes: Vec<String> = Vec::new();
    for f in funds {
        if !asset_classes.contains(&f.asset_class) {
            asset_classes.push(f.asset_class.clone());
        }
    }

    SponsorWithMetrics {
        sponsor: sponsor.clone(),
        fund_count,
        total_nav,
        total_commitment,
        tvpi,
        net_irr,
        asset_classes,
        company_count,
    }
}

// ---------------------------------------------------------------------------
// Read handlers
// ---------------------------------------------------------------------------

/// `GET /api/v1/entities`
///
/// Returns every entity in the requesting user's organization.
#[get("/entities")]
pub async fn get_entities(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let user = current_user(&req, &state.db).await?;
    let entities: Vec<crate::models::portfolio::Entity> = state
        .db
        .select(
            "entities",
            &format!("organization_id=eq.{}&order=created_at.asc", user.organization_id),
        )
        .await?;

    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "no-store"))
        .json(entities))
}

/// `GET /api/v1/sponsors?entity={id}`
#[get("/sponsors")]
pub async fn get_sponsors(
    req: HttpRequest,
    query: web::Query<SponsorFilter>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let user = current_user(&req, &state.db).await?;
    let entity_filter = query.entity.as_deref();

    let sponsors: Vec<Sponsor> = state
        .db
        .select(
            "sponsors",
            &format!("organization_id=eq.{}", user.organization_id),
        )
        .await?;

    // Pull all funds for the org once and bucket them per sponsor.
    let funds_filter = match entity_filter {
        Some(eid) => format!("organization_id=eq.{}&entity_id=eq.{eid}", user.organization_id),
        None => format!("organization_id=eq.{}", user.organization_id),
    };
    let funds: Vec<Fund> = state.db.select("funds", &funds_filter).await?;

    let result: Vec<SponsorWithMetrics> = sponsors
        .iter()
        .filter_map(|sponsor| {
            let qualifying: Vec<&Fund> = funds
                .iter()
                .filter(|f| f.sponsor_id == sponsor.id)
                .collect();
            if entity_filter.is_some() && qualifying.is_empty() {
                return None;
            }
            Some(aggregate(sponsor, &qualifying))
        })
        .collect();

    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "no-store"))
        .json(result))
}

/// `GET /api/v1/sponsors/{id}`
#[get("/sponsors/{id}")]
pub async fn get_sponsor_by_id(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let user = current_user(&req, &state.db).await?;
    let sponsor_id = path.into_inner();

    let sponsor: Sponsor = state
        .db
        .select_one(
            "sponsors",
            &format!(
                "id=eq.{sponsor_id}&organization_id=eq.{}",
                user.organization_id
            ),
        )
        .await?;

    let funds: Vec<Fund> = state
        .db
        .select(
            "funds",
            &format!(
                "sponsor_id=eq.{sponsor_id}&organization_id=eq.{}",
                user.organization_id
            ),
        )
        .await?;

    let qualifying: Vec<&Fund> = funds.iter().collect();
    let result = aggregate(&sponsor, &qualifying);

    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "no-store"))
        .json(result))
}

/// `GET /api/v1/funds?entity={id}&sponsor={id}`
#[get("/funds")]
pub async fn get_funds(
    req: HttpRequest,
    query: web::Query<FundFilter>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let user = current_user(&req, &state.db).await?;

    let mut filter = format!("organization_id=eq.{}", user.organization_id);
    if let Some(eid) = query.entity.as_deref() {
        filter.push_str(&format!("&entity_id=eq.{eid}"));
    }
    if let Some(sid) = query.sponsor.as_deref() {
        filter.push_str(&format!("&sponsor_id=eq.{sid}"));
    }

    let funds: Vec<Fund> = state.db.select("funds", &filter).await?;
    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "no-store"))
        .json(funds))
}

/// `GET /api/v1/funds/{id}`
#[get("/funds/{id}")]
pub async fn get_fund_by_id(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let user = current_user(&req, &state.db).await?;
    let fund_id = path.into_inner();

    let fund: Fund = state
        .db
        .select_one(
            "funds",
            &format!(
                "id=eq.{fund_id}&organization_id=eq.{}",
                user.organization_id
            ),
        )
        .await?;

    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "no-store"))
        .json(fund))
}

// ---------------------------------------------------------------------------
// Write handlers
// ---------------------------------------------------------------------------

const NAME_MAX: usize = 200;

fn is_valid_initials(s: &str) -> bool {
    let trimmed_len = s.trim().chars().count();
    (1..=4).contains(&trimmed_len)
        && s.chars().all(|c| c.is_alphanumeric() || c.is_whitespace())
}

fn is_valid_color(s: &str) -> bool {
    if !s.starts_with('#') {
        return false;
    }
    let hex = &s[1..];
    matches!(hex.len(), 3 | 6) && hex.chars().all(|c| c.is_ascii_hexdigit())
}

/// `POST /api/v1/sponsors`
#[post("/sponsors")]
pub async fn create_sponsor(
    req: HttpRequest,
    body: web::Json<CreateSponsorPayload>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let user = current_user(&req, &state.db).await?;

    let name = body.name.trim().to_string();
    let initials = body.initials.trim().to_uppercase();
    let country = body.country.trim().to_string();
    let color = body.color.trim().to_string();

    if name.is_empty() || name.len() > NAME_MAX {
        return Err(AppError::BadRequest(format!(
            "name must be 1–{NAME_MAX} characters"
        )));
    }
    if !is_valid_initials(&initials) {
        return Err(AppError::BadRequest(
            "initials must be 1–4 alphanumeric characters".to_string(),
        ));
    }
    if country.len() > NAME_MAX {
        return Err(AppError::BadRequest(format!(
            "country must be at most {NAME_MAX} characters"
        )));
    }
    if !is_valid_color(&color) {
        return Err(AppError::BadRequest(
            "color must be a hex string like #8B7BD8 or #ABC".to_string(),
        ));
    }

    let id = format!("s-{}", uuid::Uuid::new_v4().simple());

    let sponsor: Sponsor = state
        .db
        .insert(
            "sponsors",
            &NewSponsor {
                id: &id,
                organization_id: &user.organization_id,
                name: &name,
                initials: &initials,
                country: &country,
                color: &color,
            },
        )
        .await?;

    Ok(HttpResponse::Created().json(sponsor))
}

/// `DELETE /api/v1/sponsors/{id}`
#[delete("/sponsors/{id}")]
pub async fn delete_sponsor(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let user = current_user(&req, &state.db).await?;
    let sponsor_id = path.into_inner();

    let attached = state
        .db
        .count(
            "funds",
            &format!(
                "sponsor_id=eq.{sponsor_id}&organization_id=eq.{}",
                user.organization_id
            ),
        )
        .await?;
    if attached > 0 {
        return Err(AppError::Conflict(format!(
            "Cannot delete sponsor '{sponsor_id}': {attached} fund(s) still reference it."
        )));
    }

    state
        .db
        .delete(
            "sponsors",
            &format!(
                "id=eq.{sponsor_id}&organization_id=eq.{}",
                user.organization_id
            ),
        )
        .await?;

    Ok(HttpResponse::NoContent().finish())
}
