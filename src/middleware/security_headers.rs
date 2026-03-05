use std::{
    future::{ready, Ready},
    rc::Rc,
};

use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::{HeaderName, HeaderValue},
    Error,
};
use futures_util::future::LocalBoxFuture;

// ── Transform (factory) ──────────────────────────────────────────────────────

/// Actix-web middleware factory that injects hardened security headers into
/// every outgoing response.
///
/// Headers applied:
/// - `Content-Security-Policy` — restricts resource origins
/// - `Strict-Transport-Security` — enforces HTTPS with a 1-year max-age
/// - `X-Frame-Options` — prevents click-jacking via frame embedding
/// - `X-Content-Type-Options` — blocks MIME-type sniffing
/// - `Referrer-Policy` — limits referrer leakage
/// - `Permissions-Policy` — denies access to sensitive browser APIs
#[derive(Clone, Default)]
pub struct SecurityHeaders;

impl<S, B> Transform<S, ServiceRequest> for SecurityHeaders
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = SecurityHeadersMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SecurityHeadersMiddleware {
            service: Rc::new(service),
        }))
    }
}

// ── Middleware (per-request service) ─────────────────────────────────────────

pub struct SecurityHeadersMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = Rc::clone(&self.service);

        Box::pin(async move {
            let mut res = svc.call(req).await?.map_into_left_body();

            let headers = res.headers_mut();

            // Restrict resource loading to same origin only.
            headers.insert(
                HeaderName::from_static("content-security-policy"),
                HeaderValue::from_static("default-src 'self'"),
            );

            // Enforce HTTPS for 1 year, cascade to subdomains.
            headers.insert(
                HeaderName::from_static("strict-transport-security"),
                HeaderValue::from_static("max-age=31536000; includeSubDomains"),
            );

            // Deny embedding in frames to prevent click-jacking.
            headers.insert(
                HeaderName::from_static("x-frame-options"),
                HeaderValue::from_static("DENY"),
            );

            // Prevent MIME-type sniffing, which can enable XSS via crafted
            // uploads interpreted as executable content.
            headers.insert(
                HeaderName::from_static("x-content-type-options"),
                HeaderValue::from_static("nosniff"),
            );

            // Limit referrer to origin + path, never cross-origin full URLs.
            headers.insert(
                HeaderName::from_static("referrer-policy"),
                HeaderValue::from_static("strict-origin-when-cross-origin"),
            );

            // Explicitly deny sensitive browser permissions.
            headers.insert(
                HeaderName::from_static("permissions-policy"),
                HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
            );

            Ok(res)
        })
    }
}
