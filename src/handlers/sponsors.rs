use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::data::portfolio::{FUNDS, SPONSORS};
use crate::errors::AppError;
use crate::models::portfolio::{Fund, Sponsor};
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// A sponsor decorated with aggregated portfolio metrics.
///
/// The `#[serde(flatten)]` on `sponsor` emits all sponsor fields at the top
/// level so the client receives a single flat object rather than a nested one.
#[derive(Debug, Serialize)]
pub struct SponsorWithMetrics {
    #[serde(flatten)]
    pub sponsor: Sponsor,
    /// Number of funds belonging to this sponsor (within the requested entity
    /// filter, if any).
    pub fund_count: usize,
    /// Sum of NAV across all qualifying funds.
    pub total_nav: f64,
    /// Sum of commitment across all qualifying funds.
    pub total_commitment: f64,
    /// Paid-in-weighted average TVPI across qualifying funds.
    pub tvpi: f64,
    /// Paid-in-weighted average net IRR across qualifying funds.
    pub net_irr: f64,
    /// Deduplicated list of asset classes across qualifying funds.
    pub asset_classes: Vec<String>,
    /// Total number of portfolio companies across qualifying funds.
    pub company_count: usize,
}

// ---------------------------------------------------------------------------
// Query parameter structs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SponsorFilter {
    /// Optional entity ID to restrict results to sponsors that have at least
    /// one fund associated with that entity.
    pub entity: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FundFilter {
    /// Filter funds by entity ID.
    pub entity: Option<String>,
    /// Filter funds by sponsor ID.
    pub sponsor: Option<String>,
}

// ---------------------------------------------------------------------------
// Aggregation helpers
// ---------------------------------------------------------------------------

/// Build a [`SponsorWithMetrics`] for a given sponsor from a pre-filtered
/// slice of that sponsor's funds.
fn aggregate(sponsor: &Sponsor, funds: &[&Fund]) -> SponsorWithMetrics {
    let fund_count = funds.len();
    let total_nav: f64 = funds.iter().map(|f| f.nav).sum();
    let total_commitment: f64 = funds.iter().map(|f| f.commitment).sum();
    let company_count: usize = funds.iter().map(|f| f.companies.len()).sum();

    // Paid-in-weighted average TVPI and net IRR.
    let total_paid_in: f64 = funds.iter().map(|f| f.paid_in).sum();
    let (tvpi, net_irr) = if total_paid_in > 0.0 {
        let w_tvpi: f64 = funds.iter().map(|f| f.tvpi * f.paid_in).sum();
        let w_irr: f64 = funds.iter().map(|f| f.net_irr * f.paid_in).sum();
        (w_tvpi / total_paid_in, w_irr / total_paid_in)
    } else {
        (0.0, 0.0)
    };

    // Deduplicate asset classes while preserving first-seen order.
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
// Handlers
// ---------------------------------------------------------------------------

/// `GET /api/v1/entities`
///
/// Returns all legal entities in the portfolio.  Reads from the mutable
/// [`AppState::entities`] so newly-created entities show up here as well.
#[get("/entities")]
pub async fn get_entities(
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let entities = state.entities.read().await.clone();
    Ok(HttpResponse::Ok()
        // Keep cache short — entities are now mutable, but `revalidator`
        // on the client invalidates explicitly so the staleness window is
        // bounded and acceptable.
        .insert_header(("Cache-Control", "public, max-age=10, stale-while-revalidate=30"))
        .json(entities))
}

/// `GET /api/v1/sponsors?entity={id}`
///
/// Returns all sponsors with aggregated portfolio metrics.  When `entity` is
/// provided, only sponsors that have at least one fund associated with that
/// entity are returned, and metrics are computed using only those funds.
#[get("/sponsors")]
pub async fn get_sponsors(
    query: web::Query<SponsorFilter>,
) -> Result<HttpResponse, AppError> {
    let entity_filter = query.entity.as_deref();

    let result: Vec<SponsorWithMetrics> = SPONSORS
        .iter()
        .filter_map(|sponsor| {
            // Collect qualifying funds for this sponsor.
            let funds: Vec<&Fund> = FUNDS
                .iter()
                .filter(|f| {
                    f.sponsor_id == sponsor.id
                        && entity_filter.map_or(true, |eid| f.entity_id == eid)
                })
                .collect();

            // Exclude sponsors with no qualifying funds when filtering by entity.
            if entity_filter.is_some() && funds.is_empty() {
                return None;
            }

            Some(aggregate(sponsor, &funds))
        })
        .collect();

    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "public, max-age=60, stale-while-revalidate=300"))
        .json(result))
}

/// `GET /api/v1/sponsors/{id}`
///
/// Returns a single sponsor with aggregated portfolio metrics (across all
/// entities).  Returns 404 when the sponsor ID does not exist.
#[get("/sponsors/{id}")]
pub async fn get_sponsor_by_id(
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let sponsor_id = path.into_inner();

    let sponsor = SPONSORS
        .iter()
        .find(|s| s.id == sponsor_id)
        .ok_or_else(|| AppError::NotFound(format!("Sponsor '{}' not found.", sponsor_id)))?;

    let funds: Vec<&Fund> = FUNDS.iter().filter(|f| f.sponsor_id == sponsor_id).collect();
    let result = aggregate(sponsor, &funds);

    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "public, max-age=60, stale-while-revalidate=300"))
        .json(result))
}

/// `GET /api/v1/funds?entity={id}&sponsor={id}`
///
/// Returns all funds, optionally filtered by entity ID and/or sponsor ID.
/// Filters are applied with AND semantics when both are present.
#[get("/funds")]
pub async fn get_funds(
    query: web::Query<FundFilter>,
) -> Result<HttpResponse, AppError> {
    let entity_filter = query.entity.as_deref();
    let sponsor_filter = query.sponsor.as_deref();

    let funds: Vec<&Fund> = FUNDS
        .iter()
        .filter(|f| {
            entity_filter.map_or(true, |eid| f.entity_id == eid)
                && sponsor_filter.map_or(true, |sid| f.sponsor_id == sid)
        })
        .collect();

    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "public, max-age=60, stale-while-revalidate=300"))
        .json(funds))
}

/// `GET /api/v1/funds/{id}`
///
/// Returns the full fund detail record, including the embedded `companies`,
/// `nav_history`, and `cashflows` arrays.  Returns 404 when the fund ID does
/// not exist.
#[get("/funds/{id}")]
pub async fn get_fund_by_id(
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let fund_id = path.into_inner();

    let fund = FUNDS
        .iter()
        .find(|f| f.id == fund_id)
        .ok_or_else(|| AppError::NotFound(format!("Fund '{}' not found.", fund_id)))?;

    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "public, max-age=60, stale-while-revalidate=300"))
        .json(fund))
}
