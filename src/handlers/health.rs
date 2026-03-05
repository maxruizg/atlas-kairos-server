use actix_web::{get, HttpResponse};
use chrono::Utc;
use serde_json::json;

/// `GET /api/v1/health`
///
/// Lightweight liveness probe.  Kubernetes / load-balancers hit this
/// endpoint to decide whether traffic should be routed to this instance.
/// The response is intentionally minimal — no database round-trips — so
/// it always returns quickly.
#[get("/health")]
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "status": "ok",
        "timestamp": Utc::now().to_rfc3339(),
    }))
}
