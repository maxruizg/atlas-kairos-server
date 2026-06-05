use actix_multipart::Multipart;
use actix_web::{get, patch, post, web, HttpRequest, HttpResponse};
use futures_util::StreamExt;
use serde::Deserialize;
use std::path::Path;

use crate::errors::AppError;
use crate::handlers::auth::current_user;
use crate::models::document::{Document, UpdateStatusRequest};
use crate::state::AppState;

const VALID_STATUSES: &[&str] = &[
    "Uploaded",
    "Extracted",
    "Needs Review",
    "Approved",
    "Posted",
];

#[derive(Debug, Deserialize)]
pub struct DocumentFilter {
    pub status: Option<String>,
}

/// `GET /api/v1/documents?status=`
#[get("/documents")]
pub async fn list_documents(
    req: HttpRequest,
    query: web::Query<DocumentFilter>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let user = current_user(&req, &state.db).await?;

    let mut filter = format!("organization_id=eq.{}", user.organization_id);
    if let Some(status) = query.status.as_deref() {
        if !status.is_empty() {
            filter.push_str(&format!("&status=eq.{status}"));
        }
    }
    filter.push_str("&order=created_at.desc");

    let docs: Vec<Document> = state.db.select("documents", &filter).await?;
    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "no-store"))
        .json(docs))
}

#[derive(serde::Serialize)]
struct StatusPatch<'a> {
    status: &'a str,
}

/// `PATCH /api/v1/documents/{id}/status`
#[patch("/documents/{id}/status")]
pub async fn update_document_status(
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<UpdateStatusRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let user = current_user(&req, &state.db).await?;
    let doc_id = path.into_inner();

    if !VALID_STATUSES.contains(&body.status.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Invalid status '{}'. Must be one of: {}",
            body.status,
            VALID_STATUSES.join(", ")
        )));
    }

    let updated: Document = state
        .db
        .update(
            "documents",
            &format!("id=eq.{doc_id}&organization_id=eq.{}", user.organization_id),
            &StatusPatch {
                status: &body.status,
            },
        )
        .await?;

    Ok(HttpResponse::Ok().json(updated))
}

/// Maximum file size: 50 MiB.
const MAX_FILE_SIZE: usize = 50 * 1024 * 1024;
const ALLOWED_EXTENSIONS: &[&str] = &["pdf", "xlsx"];

fn format_file_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// `POST /api/v1/documents/upload`
///
/// File bytes still go to the local `uploads/` directory.  The metadata
/// row is persisted to Supabase (storage of the bytes themselves in
/// Supabase Storage is a follow-up commit).
#[post("/documents/upload")]
pub async fn upload_document(
    req: HttpRequest,
    mut payload: Multipart,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let user = current_user(&req, &state.db).await?;

    let mut file_data: Option<(String, Vec<u8>)> = None;
    let mut doc_type: Option<String> = None;
    let mut fund: Option<String> = None;

    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| AppError::BadRequest(format!("Multipart error: {e}")))?;

        let field_name = field
            .name()
            .ok_or_else(|| AppError::BadRequest("Missing field name".to_string()))?
            .to_string();

        match field_name.as_str() {
            "file" => {
                let cd = field
                    .content_disposition()
                    .ok_or_else(|| AppError::BadRequest("Missing content disposition".to_string()))?;
                let filename = cd
                    .get_filename()
                    .ok_or_else(|| AppError::BadRequest("Missing filename".to_string()))?
                    .to_string();

                let mut bytes = Vec::new();
                while let Some(chunk) = field.next().await {
                    let chunk =
                        chunk.map_err(|e| AppError::BadRequest(format!("Read error: {e}")))?;
                    if bytes.len() + chunk.len() > MAX_FILE_SIZE {
                        return Err(AppError::BadRequest(format!(
                            "File exceeds maximum size of {} MB",
                            MAX_FILE_SIZE / (1024 * 1024)
                        )));
                    }
                    bytes.extend_from_slice(&chunk);
                }

                file_data = Some((filename, bytes));
            }
            "doc_type" => {
                let mut value = Vec::new();
                while let Some(chunk) = field.next().await {
                    let chunk =
                        chunk.map_err(|e| AppError::BadRequest(format!("Read error: {e}")))?;
                    value.extend_from_slice(&chunk);
                }
                let s = String::from_utf8(value)
                    .map_err(|_| AppError::BadRequest("Invalid UTF-8 in doc_type".to_string()))?;
                if s.is_empty() || s.len() > 200 {
                    return Err(AppError::BadRequest(
                        "doc_type must be 1–200 characters".to_string(),
                    ));
                }
                doc_type = Some(s);
            }
            "fund" => {
                let mut value = Vec::new();
                while let Some(chunk) = field.next().await {
                    let chunk =
                        chunk.map_err(|e| AppError::BadRequest(format!("Read error: {e}")))?;
                    value.extend_from_slice(&chunk);
                }
                let s = String::from_utf8(value)
                    .map_err(|_| AppError::BadRequest("Invalid UTF-8 in fund".to_string()))?;
                if s.is_empty() || s.len() > 200 {
                    return Err(AppError::BadRequest(
                        "fund must be 1–200 characters".to_string(),
                    ));
                }
                fund = Some(s);
            }
            _ => {}
        }
    }

    let (filename, bytes) =
        file_data.ok_or_else(|| AppError::BadRequest("Missing 'file' field".to_string()))?;
    let doc_type =
        doc_type.ok_or_else(|| AppError::BadRequest("Missing 'doc_type' field".to_string()))?;
    let fund = fund.ok_or_else(|| AppError::BadRequest("Missing 'fund' field".to_string()))?;

    let ext = Path::new(&filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    if !ALLOWED_EXTENSIONS.contains(&ext.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Unsupported file type '.{ext}'. Allowed: {}",
            ALLOWED_EXTENSIONS.join(", ")
        )));
    }

    let file_id = uuid::Uuid::new_v4().to_string();
    let stored_name = format!("{file_id}.{ext}");
    let upload_path = Path::new("uploads").join(&stored_name);

    tokio::fs::write(&upload_path, &bytes)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to save file: {e}")))?;

    let size_str = format_file_size(bytes.len());
    let today = chrono::Utc::now().format("%b %d, %Y").to_string();

    let document = Document {
        id: file_id,
        organization_id: user.organization_id.clone(),
        name: filename,
        doc_type,
        fund,
        status: "Uploaded".to_string(),
        confidence: None,
        date: today,
        size: size_str,
        fields: 0,
        extracted: 0,
        sponsor_id: None,
        fund_id: None,
    };

    let inserted: Document = state.db.insert("documents", &document).await?;
    Ok(HttpResponse::Created().json(inserted))
}
