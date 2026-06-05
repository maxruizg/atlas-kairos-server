use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

/// Structured error response body returned to clients.
///
/// Keeping the envelope flat (`{"error": "…"}`) makes it trivial for
/// frontend consumers to check `response.error` without additional
/// destructuring.
#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

/// All application-level errors that can be converted into HTTP responses.
///
/// Variants deliberately avoid embedding internal details (stack traces,
/// SQL errors) in the message — callers should log those separately and
/// pass only a safe, human-readable string to the constructor.
#[derive(Debug)]
pub enum AppError {
    /// 404 — the requested resource does not exist.
    NotFound(String),
    /// 400 — the request is syntactically or semantically invalid.
    BadRequest(String),
    /// 401 — the request lacks a valid session.
    Unauthorized(String),
    /// 409 — the request conflicts with the current state of the resource
    /// (e.g. attempting to delete an entity that still has funds attached).
    Conflict(String),
    /// 500 — an unexpected internal failure occurred.
    Internal(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "{}", msg),
            AppError::BadRequest(msg) => write!(f, "{}", msg),
            AppError::Unauthorized(msg) => write!(f, "{}", msg),
            AppError::Conflict(msg) => write!(f, "{}", msg),
            AppError::Internal(msg) => write!(f, "{}", msg),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let body = ErrorBody {
            error: self.to_string(),
        };

        match self {
            AppError::NotFound(_) => HttpResponse::NotFound().json(body),
            AppError::BadRequest(_) => HttpResponse::BadRequest().json(body),
            AppError::Unauthorized(_) => HttpResponse::Unauthorized().json(body),
            AppError::Conflict(_) => HttpResponse::Conflict().json(body),
            // Never surface internal details — log them at the call site
            // and return a generic message to the client.
            AppError::Internal(_) => {
                log::error!("Internal server error: {}", self);
                HttpResponse::InternalServerError().json(ErrorBody {
                    error: "An internal server error occurred.".to_string(),
                })
            }
        }
    }
}
