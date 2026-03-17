use actix_web::{post, web, HttpResponse};

use crate::models::auth::{AuthResponse, LoginRequest, SignupRequest};

/// `POST /api/v1/auth/login`
#[post("/auth/login")]
pub async fn login(body: web::Json<LoginRequest>) -> HttpResponse {
    let req = body.into_inner();

    if req.email.is_empty() || req.password.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Email and password are required"
        }));
    }
    if req.email.len() > 200 || req.password.len() > 128 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Input exceeds maximum length"
        }));
    }

    HttpResponse::Ok().json(AuthResponse {
        success: true,
        token: "demo".to_string(),
    })
}

/// `POST /api/v1/auth/signup`
#[post("/auth/signup")]
pub async fn signup(body: web::Json<SignupRequest>) -> HttpResponse {
    let req = body.into_inner();

    if req.name.is_empty() || req.email.is_empty() || req.password.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "All fields are required"
        }));
    }
    if req.name.len() > 200 || req.email.len() > 200 || req.password.len() > 128 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Input exceeds maximum length"
        }));
    }
    if req.password != req.confirm_password {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Passwords do not match"
        }));
    }

    HttpResponse::Ok().json(AuthResponse {
        success: true,
        token: "demo".to_string(),
    })
}

/// `POST /api/v1/auth/logout`
#[post("/auth/logout")]
pub async fn logout() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({ "success": true }))
}
