use actix_cors::Cors;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{http::Method, middleware::Compress, web, App, HttpServer, ResponseError};
use std::sync::Arc;

mod config;
mod data;
mod errors;
mod handlers;
mod middleware;
mod models;
mod routes;
mod state;

use config::Config;
use middleware::security_headers::SecurityHeaders;
use state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load `.env` if present.  Errors are silently ignored — in production
    // the variables are injected directly into the process environment.
    let _ = dotenvy::dotenv();

    // Initialise structured logging.  The `RUST_LOG` env var controls
    // verbosity (e.g. `RUST_LOG=info,kairos_backend=debug`).
    env_logger::init();

    let cfg = Config::from_env();
    let bind_addr = cfg.bind_addr();

    // Ensure the uploads directory exists.
    std::fs::create_dir_all("uploads").expect("Failed to create uploads/ directory");

    log::info!(
        "Starting Atlas backend on {} (FRONTEND_URL={})",
        bind_addr,
        cfg.frontend_url
    );

    // Shared application state — wrapped in Arc so cloning for each worker
    // is cheap (clones the pointer, not the data).
    let app_state = Arc::new(AppState::new());

    // Rate-limiter: allow a burst of 30 requests, then replenish at 2/s.
    // This translates to 120 req/min sustained with a short-burst allowance.
    // `finish()` returns None only if burst_size or the period is zero.
    let governor_conf = GovernorConfigBuilder::default()
        .requests_per_second(2)
        .burst_size(30)
        .finish()
        .expect("Invalid governor configuration: burst_size and requests_per_second must be > 0");

    // Capture CORS origin before moving cfg into the closure.
    let frontend_url = cfg.frontend_url.clone();

    HttpServer::new(move || {
        // ── CORS ────────────────────────────────────────────────────────────
        // Strict allowlist — only the configured frontend origin may call
        // the API.  Credentials mode is enabled for cookie-based auth if
        // added later; the explicit allowlist is required in that case.
        let cors = Cors::default()
            .allowed_origin(&frontend_url)
            .allowed_methods(vec![
                Method::GET,
                Method::POST,
                Method::PATCH,
                Method::OPTIONS,
            ])
            .allowed_header(actix_web::http::header::CONTENT_TYPE)
            // Mirror the request origin in the response (required for
            // credentialed cross-origin requests in strict CORS mode).
            .supports_credentials()
            .max_age(3600);

        App::new()
            // Shared state — Actix wraps this in an Arc internally when
            // created with `web::Data::from`.
            .app_data(web::Data::from(app_state.clone()))
            // Reject oversized JSON bodies early (protect against DoS).
            .app_data(
                web::JsonConfig::default()
                    .limit(65_536) // 64 KiB
                    .error_handler(|err, _req| {
                        let response = errors::AppError::BadRequest(err.to_string())
                            .error_response();
                        actix_web::error::InternalError::from_response(err, response).into()
                    }),
            )
            // Allow multipart uploads up to 50 MiB.
            .app_data(web::PayloadConfig::default().limit(50 * 1024 * 1024))
            // Reject oversized query strings.
            .app_data(
                web::QueryConfig::default().error_handler(|err, _req| {
                    let response = errors::AppError::BadRequest(err.to_string())
                        .error_response();
                    actix_web::error::InternalError::from_response(err, response).into()
                }),
            )
            // ── Middleware stack (outermost = runs last on request / first on response) ──
            // Security headers on every response.
            .wrap(SecurityHeaders)
            // Gzip / brotli response compression.
            .wrap(Compress::default())
            // CORS pre-flight and header injection.
            .wrap(cors)
            // Rate limiting — placed after CORS so pre-flight OPTIONS
            // requests are also subject to rate limits.
            .wrap(Governor::new(&governor_conf))
            // ── Routes ──────────────────────────────────────────────────────
            .configure(routes::configure)
    })
    .bind(&bind_addr)?
    .run()
    .await
}
