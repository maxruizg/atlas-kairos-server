use actix_web::{delete, post, web, HttpResponse};
use serde::Deserialize;

use crate::data::portfolio::FUNDS;
use crate::errors::AppError;
use crate::models::portfolio::Entity;
use crate::state::AppState;

/// Request body for `POST /api/v1/entities`.
#[derive(Debug, Deserialize)]
pub struct CreateEntityPayload {
    pub name: String,
    pub short: String,
    pub nav: f64,
}

const NAME_MAX: usize = 200;
const SHORT_MIN: usize = 2;
const SHORT_MAX: usize = 6;

fn is_valid_short(s: &str) -> bool {
    s.len() >= SHORT_MIN
        && s.len() <= SHORT_MAX
        && s.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
}

/// `POST /api/v1/entities`
///
/// Creates a new legal entity.
///
/// # Validation
/// - `name` must be 1–200 characters (trimmed).
/// - `short` must be 2–6 chars, ASCII uppercase / digits, and unique.
/// - `nav` must be a finite, non-negative number.
#[post("/entities")]
pub async fn create_entity(
    body: web::Json<CreateEntityPayload>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let name = body.name.trim().to_string();
    let short = body.short.trim().to_uppercase();
    let nav = body.nav;

    if name.is_empty() || name.len() > NAME_MAX {
        return Err(AppError::BadRequest(format!(
            "name must be 1–{NAME_MAX} characters"
        )));
    }
    if !is_valid_short(&short) {
        return Err(AppError::BadRequest(format!(
            "short must be {SHORT_MIN}–{SHORT_MAX} uppercase letters or digits"
        )));
    }
    if !nav.is_finite() || nav < 0.0 {
        return Err(AppError::BadRequest(
            "nav must be a finite, non-negative number".to_string(),
        ));
    }

    let mut entities = state.entities.write().await;

    if entities.iter().any(|e| e.short.eq_ignore_ascii_case(&short)) {
        return Err(AppError::Conflict(format!(
            "An entity with short code '{short}' already exists"
        )));
    }

    let entity = Entity {
        id: format!("e-{}", uuid::Uuid::new_v4().simple()),
        name,
        short,
        nav,
    };

    entities.push(entity.clone());
    Ok(HttpResponse::Created().json(entity))
}

/// `DELETE /api/v1/entities/{id}`
///
/// Removes an entity.  Refuses with 409 Conflict if any fund still
/// references the entity (cascade-delete is intentionally NOT supported —
/// the user must reassign or remove dependent funds first).
#[delete("/entities/{id}")]
pub async fn delete_entity(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let entity_id = path.into_inner();

    // Block deletion when funds still reference the entity.
    let attached = FUNDS.iter().filter(|f| f.entity_id == entity_id).count();
    if attached > 0 {
        return Err(AppError::Conflict(format!(
            "Cannot delete entity '{entity_id}': {attached} fund(s) still reference it. Reassign or remove the attached funds first."
        )));
    }

    let mut entities = state.entities.write().await;
    let before = entities.len();
    entities.retain(|e| e.id != entity_id);

    if entities.len() == before {
        return Err(AppError::NotFound(format!(
            "Entity '{entity_id}' not found."
        )));
    }

    Ok(HttpResponse::NoContent().finish())
}
