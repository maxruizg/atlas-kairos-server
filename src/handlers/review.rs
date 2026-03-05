use actix_web::{get, post, web, HttpResponse};

use crate::errors::AppError;
use crate::models::review::FieldActionRequest;
use crate::state::AppState;

/// `GET /api/v1/review/{document_id}`
///
/// Returns the review document with all extracted fields.
///
/// Currently the system holds a single review document seeded at startup.
/// If the requested document ID does not match it, a 404 is returned so
/// the API contract is preserved when multiple documents are added later.
#[get("/review/{document_id}")]
pub async fn get_review_document(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let document_id = path.into_inner();
    let review = state.review.read().await;

    if review.id != document_id {
        return Err(AppError::NotFound(format!(
            "Review document '{}' not found.",
            document_id
        )));
    }

    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "public, max-age=60, stale-while-revalidate=300"))
        .json(&*review))
}

/// `POST /api/v1/review/{document_id}/fields/{field_id}`
///
/// Approves or rejects an individual extracted field.
///
/// The `approved` boolean in the request body is recorded directly on the
/// field; `None` (pending) can only be set by resetting the document, not
/// via this endpoint.
///
/// # Validation
/// - The document ID must match the seeded review document (404 otherwise).
/// - The field ID must exist within that document (404 otherwise).
#[post("/review/{document_id}/fields/{field_id}")]
pub async fn action_review_field(
    path: web::Path<(String, String)>,
    body: web::Json<FieldActionRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let (document_id, field_id) = path.into_inner();
    let mut review = state.review.write().await;

    if review.id != document_id {
        return Err(AppError::NotFound(format!(
            "Review document '{}' not found.",
            document_id
        )));
    }

    let field = review
        .fields
        .iter_mut()
        .find(|f| f.id == field_id)
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "Field '{}' not found in document '{}'.",
                field_id, document_id
            ))
        })?;

    field.approved = Some(body.approved);

    log::info!(
        "Field '{}' in document '{}' marked as {}.",
        field_id,
        document_id,
        if body.approved { "approved" } else { "rejected" }
    );

    Ok(HttpResponse::Ok().json(&*field))
}
