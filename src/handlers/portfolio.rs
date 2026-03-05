use actix_web::{get, web, HttpResponse};
use serde::Deserialize;

use crate::data::portfolio::{INVESTMENTS, KPIS, THEMES};
use crate::errors::AppError;

/// Query parameters accepted by the investments endpoint.
#[derive(Debug, Deserialize)]
pub struct InvestmentFilter {
    /// Filter by investment type.  Accepted values (case-insensitive):
    /// `all` | `vc` | `pe` | `direct` | `real_assets`
    pub filter: Option<String>,
}

/// `GET /api/v1/portfolio/kpis`
///
/// Returns the five top-level portfolio KPI cards.
/// Cache-Control is set by the route scope in `routes.rs`.
#[get("/portfolio/kpis")]
pub async fn get_kpis() -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "public, max-age=60, stale-while-revalidate=300"))
        .json(&*KPIS))
}

/// `GET /api/v1/portfolio/investments?filter=`
///
/// Returns the investment list, optionally filtered by type.
/// The match is case-insensitive; `real_assets` maps to `"Real Assets"`.
#[get("/portfolio/investments")]
pub async fn get_investments(
    query: web::Query<InvestmentFilter>,
) -> Result<HttpResponse, AppError> {
    let filter = query
        .filter
        .as_deref()
        .unwrap_or("all")
        .to_lowercase();

    let investments: Vec<_> = INVESTMENTS
        .iter()
        .filter(|inv| match filter.as_str() {
            "all" | "" => true,
            "vc" => inv.investment_type.eq_ignore_ascii_case("VC"),
            "pe" => inv.investment_type.eq_ignore_ascii_case("PE"),
            "direct" => inv.investment_type.eq_ignore_ascii_case("Direct"),
            "real_assets" => inv.investment_type.eq_ignore_ascii_case("Real Assets"),
            unknown => {
                // Return a bad-request rather than silently returning empty
                // results, which is harder to debug from the frontend.
                log::warn!("Unknown investment filter value: {}", unknown);
                false
            }
        })
        .collect();

    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "public, max-age=60, stale-while-revalidate=300"))
        .json(investments))
}

/// `GET /api/v1/portfolio/themes`
///
/// Returns the theme allocation breakdown.
#[get("/portfolio/themes")]
pub async fn get_themes() -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "public, max-age=60, stale-while-revalidate=300"))
        .json(&*THEMES))
}
