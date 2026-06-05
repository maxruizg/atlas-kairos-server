use actix_web::web;

use crate::handlers::{
    auth::{login, logout, me, signup},
    copilot::{get_history, get_suggestions, post_query},
    documents::{list_documents, update_document_status, upload_document},
    entities::{create_entity, delete_entity, update_entity},
    funds::{create_company, create_fund, delete_company, delete_fund, update_fund_entity},
    health::health_check,
    organization::{get_organization, update_organization},
    portfolio::{get_investments, get_kpis, get_themes},
    review::{action_review_field, get_review_document},
    sponsors::{
        create_sponsor, delete_sponsor, get_entities, get_fund_by_id, get_funds,
        get_sponsor_by_id, get_sponsors,
    },
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
            .service(create_entity)
            .service(update_entity)
            .service(delete_entity)
            .service(get_sponsors)
            .service(create_sponsor)
            .service(delete_sponsor)
            .service(get_sponsor_by_id)
            .service(get_funds)
            .service(create_fund)
            .service(delete_fund)
            .service(update_fund_entity)
            .service(create_company)
            .service(delete_company)
            .service(get_fund_by_id)
            // Organization (tenant settings + onboarding flag)
            .service(get_organization)
            .service(update_organization)
            // Auth
            .service(login)
            .service(signup)
            .service(logout)
            .service(me),
    );
}
