use actix_web::web;

use crate::handlers::{
    auth::{login, logout, signup},
    copilot::{get_history, get_suggestions, post_query},
    documents::{list_documents, update_document_status, upload_document},
    health::health_check,
    portfolio::{get_investments, get_kpis, get_themes},
    review::{action_review_field, get_review_document},
    sponsors::{get_entities, get_fund_by_id, get_funds, get_sponsor_by_id, get_sponsors},
};

/// Register all application routes under the `/api/v1` scope.
///
/// Every handler is a separate function registered with its own HTTP-method
/// macro (`#[get]`, `#[post]`, `#[patch]`), so the route table here is
/// intentionally thin — it only wires up the scope.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            // Health
            .service(health_check)
            // Portfolio (read-only, cached at the handler level)
            .service(get_kpis)
            .service(get_investments)
            .service(get_themes)
            // Documents
            .service(list_documents)
            .service(update_document_status)
            .service(upload_document)
            // Review
            .service(get_review_document)
            .service(action_review_field)
            // Copilot
            .service(get_history)
            .service(get_suggestions)
            .service(post_query)
            // Phase 1 — Entities, Sponsors, Funds
            .service(get_entities)
            .service(get_sponsors)
            .service(get_sponsor_by_id)
            .service(get_funds)
            .service(get_fund_by_id)
            // Auth
            .service(login)
            .service(signup)
            .service(logout),
    );
}
