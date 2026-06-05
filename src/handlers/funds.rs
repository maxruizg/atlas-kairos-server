use actix_web::{delete, patch, post, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::errors::AppError;
use crate::handlers::auth::current_user;
use crate::models::portfolio::{Entity, Fund, PortfolioCompany, Sponsor};
use crate::state::AppState;

const NAME_MAX: usize = 200;
const VALID_COMPANY_STATUSES: &[&str] = &["Active", "Realized", "Written Off"];

#[derive(Debug, Deserialize)]
pub struct UpdateFundEntityPayload {
    pub entity_id: String,
}

/// PostgREST patch payload — only the entity_id changes when reassigning.
#[derive(Serialize)]
struct EntityPatch<'a> {
    entity_id: &'a str,
}

/// PostgREST patch payload for the companies JSONB column.
#[derive(Serialize)]
struct CompaniesPatch<'a> {
    companies: &'a [PortfolioCompany],
}

/// `POST /api/v1/funds`
#[post("/funds")]
pub async fn create_fund(
    req: HttpRequest,
    body: web::Json<Fund>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let user = current_user(&req, &state.db).await?;
    let mut fund = body.into_inner();

    let name = fund.name.trim().to_string();
    if name.is_empty() || name.len() > NAME_MAX {
        return Err(AppError::BadRequest(format!(
            "name must be 1–{NAME_MAX} characters"
        )));
    }
    if !(1900..=2200).contains(&fund.vintage) {
        return Err(AppError::BadRequest(
            "vintage must be a four-digit year between 1900 and 2200".to_string(),
        ));
    }

    // Foreign-key checks: sponsor + entity must exist in the user's org.
    let _sponsor: Sponsor = state
        .db
        .select_one(
            "sponsors",
            &format!(
                "id=eq.{}&organization_id=eq.{}",
                fund.sponsor_id, user.organization_id
            ),
        )
        .await
        .map_err(|_| AppError::NotFound(format!("Sponsor '{}' not found.", fund.sponsor_id)))?;

    let _entity: Entity = state
        .db
        .select_one(
            "entities",
            &format!(
                "id=eq.{}&organization_id=eq.{}",
                fund.entity_id, user.organization_id
            ),
        )
        .await
        .map_err(|_| AppError::NotFound(format!("Entity '{}' not found.", fund.entity_id)))?;

    fund.name = name;
    fund.id = format!("f-{}", uuid::Uuid::new_v4().simple());
    fund.organization_id = user.organization_id.clone();

    let created: Fund = state.db.insert("funds", &fund).await?;
    Ok(HttpResponse::Created().json(created))
}

/// `DELETE /api/v1/funds/{id}`
#[delete("/funds/{id}")]
pub async fn delete_fund(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let user = current_user(&req, &state.db).await?;
    let fund_id = path.into_inner();

    state
        .db
        .delete(
            "funds",
            &format!(
                "id=eq.{fund_id}&organization_id=eq.{}",
                user.organization_id
            ),
        )
        .await?;

    Ok(HttpResponse::NoContent().finish())
}

/// `PATCH /api/v1/funds/{id}/entity`
#[patch("/funds/{id}/entity")]
pub async fn update_fund_entity(
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<UpdateFundEntityPayload>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let user = current_user(&req, &state.db).await?;
    let fund_id = path.into_inner();
    let new_entity_id = body.entity_id.trim().to_string();

    // Confirm the new entity exists in the same org.
    let _entity: Entity = state
        .db
        .select_one(
            "entities",
            &format!(
                "id=eq.{new_entity_id}&organization_id=eq.{}",
                user.organization_id
            ),
        )
        .await
        .map_err(|_| AppError::NotFound(format!("Entity '{new_entity_id}' not found.")))?;

    let fund: Fund = state
        .db
        .update(
            "funds",
            &format!(
                "id=eq.{fund_id}&organization_id=eq.{}",
                user.organization_id
            ),
            &EntityPatch {
                entity_id: &new_entity_id,
            },
        )
        .await?;

    Ok(HttpResponse::Ok().json(fund))
}

/// `POST /api/v1/funds/{id}/companies`
#[post("/funds/{id}/companies")]
pub async fn create_company(
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<PortfolioCompany>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let user = current_user(&req, &state.db).await?;
    let fund_id = path.into_inner();
    let mut company = body.into_inner();

    // Validate the incoming company.
    let name = company.name.trim().to_string();
    if name.is_empty() || name.len() > NAME_MAX {
        return Err(AppError::BadRequest(format!(
            "name must be 1–{NAME_MAX} characters"
        )));
    }
    if !VALID_COMPANY_STATUSES.contains(&company.status.as_str()) {
        return Err(AppError::BadRequest(format!(
            "status must be one of: {}",
            VALID_COMPANY_STATUSES.join(", ")
        )));
    }
    for (label, v) in [
        ("invested", company.invested),
        ("fmv", company.fmv),
        ("moic", company.moic),
        ("own", company.own),
    ] {
        if !v.is_finite() || v < 0.0 {
            return Err(AppError::BadRequest(format!(
                "{label} must be a finite, non-negative number"
            )));
        }
    }
    if !company.irr.is_finite() {
        return Err(AppError::BadRequest(
            "irr must be a finite number".to_string(),
        ));
    }
    company.name = name;

    // Read-modify-write the fund's companies JSONB array.
    let mut fund: Fund = state
        .db
        .select_one(
            "funds",
            &format!(
                "id=eq.{fund_id}&organization_id=eq.{}",
                user.organization_id
            ),
        )
        .await?;

    if fund.companies.iter().any(|c| c.name == company.name) {
        return Err(AppError::Conflict(format!(
            "Company '{}' already exists in fund '{}'.",
            company.name, fund_id
        )));
    }
    fund.companies.push(company.clone());

    let _updated: Fund = state
        .db
        .update(
            "funds",
            &format!(
                "id=eq.{fund_id}&organization_id=eq.{}",
                user.organization_id
            ),
            &CompaniesPatch {
                companies: &fund.companies,
            },
        )
        .await?;

    Ok(HttpResponse::Created().json(company))
}

/// `DELETE /api/v1/funds/{id}/companies/{name}`
#[delete("/funds/{id}/companies/{name}")]
pub async fn delete_company(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let user = current_user(&req, &state.db).await?;
    let (fund_id, company_name) = path.into_inner();

    let mut fund: Fund = state
        .db
        .select_one(
            "funds",
            &format!(
                "id=eq.{fund_id}&organization_id=eq.{}",
                user.organization_id
            ),
        )
        .await?;

    let before = fund.companies.len();
    fund.companies.retain(|c| c.name != company_name);
    if fund.companies.len() == before {
        return Err(AppError::NotFound(format!(
            "Company '{company_name}' not found in fund '{fund_id}'."
        )));
    }

    let _updated: Fund = state
        .db
        .update(
            "funds",
            &format!(
                "id=eq.{fund_id}&organization_id=eq.{}",
                user.organization_id
            ),
            &CompaniesPatch {
                companies: &fund.companies,
            },
        )
        .await?;

    Ok(HttpResponse::NoContent().finish())
}
