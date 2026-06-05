use actix_web::{get, put, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::errors::AppError;
use crate::handlers::auth::current_user;
use crate::models::portfolio::OrganizationSettings;
use crate::state::AppState;

const NAME_MAX: usize = 200;

#[derive(Debug, Deserialize)]
pub struct UpdateOrganizationPayload {
    pub name: Option<String>,
    pub onboarded: Option<bool>,
}

/// PostgREST update payload — only the fields the caller wants to change.
#[derive(Serialize, Default)]
struct OrgPatch<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    onboarded: Option<bool>,
}

/// `GET /api/v1/organization`
#[get("/organization")]
pub async fn get_organization(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let user = current_user(&req, &state.db).await?;
    let org: OrganizationSettings = state
        .db
        .select_one("organizations", &format!("id=eq.{}", user.organization_id))
        .await?;

    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "no-store"))
        .json(org))
}

/// `PUT /api/v1/organization`
///
/// Both fields are optional so callers can patch incrementally.  Owner
/// role is required to rename; flipping `onboarded` is allowed for any
/// member (the onboarding wizard's final step).
#[put("/organization")]
pub async fn update_organization(
    req: HttpRequest,
    body: web::Json<UpdateOrganizationPayload>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let user = current_user(&req, &state.db).await?;

    let mut patch = OrgPatch::default();

    if let Some(name) = body.name.as_ref() {
        let trimmed = name.trim();
        if trimmed.is_empty() || trimmed.len() > NAME_MAX {
            return Err(AppError::BadRequest(format!(
                "name must be 1–{NAME_MAX} characters"
            )));
        }
        if user.role != "owner" {
            return Err(AppError::Unauthorized(
                "Only the organization owner can change the name".into(),
            ));
        }
        patch.name = Some(trimmed);
    }

    if let Some(onboarded) = body.onboarded {
        patch.onboarded = Some(onboarded);
    }

    if patch.name.is_none() && patch.onboarded.is_none() {
        // Nothing to update — just return the current state.
        let org: OrganizationSettings = state
            .db
            .select_one("organizations", &format!("id=eq.{}", user.organization_id))
            .await?;
        return Ok(HttpResponse::Ok().json(org));
    }

    let org: OrganizationSettings = state
        .db
        .update(
            "organizations",
            &format!("id=eq.{}", user.organization_id),
            &patch,
        )
        .await?;

    Ok(HttpResponse::Ok().json(org))
}
