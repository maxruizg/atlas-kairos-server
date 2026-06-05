use actix_web::{delete, post, put, web, HttpRequest, HttpResponse};

use crate::errors::AppError;
use crate::handlers::auth::current_user;
use crate::models::portfolio::Entity;
use crate::state::AppState;

const NAME_MAX: usize = 200;
const NOTES_MAX: usize = 4000;
const ADDRESS_MAX: usize = 200;
const SHORT_MIN: usize = 2;
const SHORT_MAX: usize = 6;

const VALID_STATUS: &[&str] = &["Active", "Dormant", "Dissolved"];
const VALID_RISK: &[&str] = &["Low", "Medium", "High"];
const VALID_FATCA: &[&str] = &[
    "US Person",
    "Active NFFE",
    "Passive NFFE",
    "FFI",
    "N/A",
];
const VALID_TAX_CLASS: &[&str] = &[
    "Pass-through",
    "Corporation",
    "Foreign",
    "Tax-exempt",
    "Other",
];

fn is_valid_short(s: &str) -> bool {
    s.len() >= SHORT_MIN
        && s.len() <= SHORT_MAX
        && s.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
}

/// Validate every optional KYC field on the [`Entity`] payload.
fn validate_kyc(entity: &Entity) -> Result<(), AppError> {
    fn capped(label: &str, v: &Option<String>, max: usize) -> Result<(), AppError> {
        if let Some(s) = v.as_ref() {
            if s.len() > max {
                return Err(AppError::BadRequest(format!(
                    "{label} must be at most {max} characters"
                )));
            }
        }
        Ok(())
    }
    fn enumerated(label: &str, v: &Option<String>, allowed: &[&str]) -> Result<(), AppError> {
        if let Some(s) = v.as_ref() {
            if !s.is_empty() && !allowed.iter().any(|a| *a == s.as_str()) {
                return Err(AppError::BadRequest(format!(
                    "{label} must be one of: {}",
                    allowed.join(", ")
                )));
            }
        }
        Ok(())
    }

    capped("entity_type", &entity.entity_type, NAME_MAX)?;
    capped("jurisdiction_country", &entity.jurisdiction_country, NAME_MAX)?;
    capped("jurisdiction_state", &entity.jurisdiction_state, NAME_MAX)?;
    capped("formation_date", &entity.formation_date, 32)?;
    capped("tax_id", &entity.tax_id, 64)?;
    capped("registration_number", &entity.registration_number, 128)?;
    enumerated("status", &entity.status, VALID_STATUS)?;

    capped("address_street", &entity.address_street, ADDRESS_MAX)?;
    capped("address_city", &entity.address_city, NAME_MAX)?;
    capped("address_state", &entity.address_state, NAME_MAX)?;
    capped("address_postal_code", &entity.address_postal_code, 32)?;
    capped("address_country", &entity.address_country, NAME_MAX)?;

    capped("contact_name", &entity.contact_name, NAME_MAX)?;
    capped("contact_title", &entity.contact_title, NAME_MAX)?;
    capped("contact_email", &entity.contact_email, NAME_MAX)?;
    capped("contact_phone", &entity.contact_phone, 64)?;
    capped("website", &entity.website, NAME_MAX)?;

    enumerated("tax_classification", &entity.tax_classification, VALID_TAX_CLASS)?;
    enumerated("fatca_status", &entity.fatca_status, VALID_FATCA)?;
    capped("kyc_verified_date", &entity.kyc_verified_date, 32)?;
    capped("kyc_verified_by", &entity.kyc_verified_by, NAME_MAX)?;
    enumerated("risk_rating", &entity.risk_rating, VALID_RISK)?;

    capped("base_currency", &entity.base_currency, 8)?;
    capped("source_of_wealth", &entity.source_of_wealth, NOTES_MAX)?;
    capped("notes", &entity.notes, NOTES_MAX)?;

    let mut total_pct = 0.0;
    for (i, bo) in entity.beneficial_owners.iter().enumerate() {
        let trimmed = bo.name.trim();
        if trimmed.is_empty() || trimmed.len() > NAME_MAX {
            return Err(AppError::BadRequest(format!(
                "beneficial_owners[{i}].name must be 1–{NAME_MAX} characters"
            )));
        }
        if !bo.ownership_pct.is_finite()
            || bo.ownership_pct < 0.0
            || bo.ownership_pct > 100.0
        {
            return Err(AppError::BadRequest(format!(
                "beneficial_owners[{i}].ownership_pct must be between 0 and 100"
            )));
        }
        if let Some(role) = bo.role.as_ref() {
            if role.len() > NAME_MAX {
                return Err(AppError::BadRequest(format!(
                    "beneficial_owners[{i}].role must be at most {NAME_MAX} characters"
                )));
            }
        }
        total_pct += bo.ownership_pct;
    }
    if !entity.beneficial_owners.is_empty() && total_pct > 100.5 {
        return Err(AppError::BadRequest(format!(
            "beneficial_owners ownership total ({total_pct:.1}%) exceeds 100%"
        )));
    }

    Ok(())
}

/// Trim, then None for empties.
fn norm(v: Option<String>) -> Option<String> {
    v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn normalise(entity: &mut Entity) {
    entity.name = entity.name.trim().to_string();
    entity.short = entity.short.trim().to_uppercase();

    entity.entity_type = norm(entity.entity_type.take());
    entity.jurisdiction_country = norm(entity.jurisdiction_country.take());
    entity.jurisdiction_state = norm(entity.jurisdiction_state.take());
    entity.formation_date = norm(entity.formation_date.take());
    entity.tax_id = norm(entity.tax_id.take());
    entity.registration_number = norm(entity.registration_number.take());
    entity.status = norm(entity.status.take());

    entity.address_street = norm(entity.address_street.take());
    entity.address_city = norm(entity.address_city.take());
    entity.address_state = norm(entity.address_state.take());
    entity.address_postal_code = norm(entity.address_postal_code.take());
    entity.address_country = norm(entity.address_country.take());

    entity.contact_name = norm(entity.contact_name.take());
    entity.contact_title = norm(entity.contact_title.take());
    entity.contact_email = norm(entity.contact_email.take());
    entity.contact_phone = norm(entity.contact_phone.take());
    entity.website = norm(entity.website.take());

    entity.tax_classification = norm(entity.tax_classification.take());
    entity.fatca_status = norm(entity.fatca_status.take());
    entity.kyc_verified_date = norm(entity.kyc_verified_date.take());
    entity.kyc_verified_by = norm(entity.kyc_verified_by.take());
    entity.risk_rating = norm(entity.risk_rating.take());

    entity.base_currency = norm(entity.base_currency.take());
    entity.source_of_wealth = norm(entity.source_of_wealth.take());
    entity.notes = norm(entity.notes.take());

    for bo in entity.beneficial_owners.iter_mut() {
        bo.name = bo.name.trim().to_string();
        bo.role = norm(bo.role.take());
    }
}

/// `POST /api/v1/entities`
#[post("/entities")]
pub async fn create_entity(
    req: HttpRequest,
    body: web::Json<Entity>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let user = current_user(&req, &state.db).await?;

    let mut entity = body.into_inner();
    normalise(&mut entity);

    if entity.name.is_empty() || entity.name.len() > NAME_MAX {
        return Err(AppError::BadRequest(format!(
            "name must be 1–{NAME_MAX} characters"
        )));
    }
    if !is_valid_short(&entity.short) {
        return Err(AppError::BadRequest(format!(
            "short must be {SHORT_MIN}–{SHORT_MAX} uppercase letters or digits"
        )));
    }
    if !entity.nav.is_finite() || entity.nav < 0.0 {
        return Err(AppError::BadRequest(
            "nav must be a finite, non-negative number".to_string(),
        ));
    }
    validate_kyc(&entity)?;

    // Server-controlled fields.
    entity.id = format!("e-{}", uuid::Uuid::new_v4().simple());
    entity.organization_id = user.organization_id.clone();

    let created: Entity = state.db.insert("entities", &entity).await?;
    Ok(HttpResponse::Created().json(created))
}

/// `PUT /api/v1/entities/{id}`
#[put("/entities/{id}")]
pub async fn update_entity(
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<Entity>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let user = current_user(&req, &state.db).await?;
    let entity_id = path.into_inner();

    let mut incoming = body.into_inner();
    normalise(&mut incoming);

    if incoming.name.is_empty() || incoming.name.len() > NAME_MAX {
        return Err(AppError::BadRequest(format!(
            "name must be 1–{NAME_MAX} characters"
        )));
    }
    if !is_valid_short(&incoming.short) {
        return Err(AppError::BadRequest(format!(
            "short must be {SHORT_MIN}–{SHORT_MAX} uppercase letters or digits"
        )));
    }
    if !incoming.nav.is_finite() || incoming.nav < 0.0 {
        return Err(AppError::BadRequest(
            "nav must be a finite, non-negative number".to_string(),
        ));
    }
    validate_kyc(&incoming)?;

    // Server-controlled — never trust the client to switch tenants or
    // rename the row id.
    incoming.id = entity_id.clone();
    incoming.organization_id = user.organization_id.clone();

    let updated: Entity = state
        .db
        .update(
            "entities",
            &format!("id=eq.{entity_id}&organization_id=eq.{}", user.organization_id),
            &incoming,
        )
        .await?;

    Ok(HttpResponse::Ok().json(updated))
}

/// `DELETE /api/v1/entities/{id}`
#[delete("/entities/{id}")]
pub async fn delete_entity(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let user = current_user(&req, &state.db).await?;
    let entity_id = path.into_inner();

    // Pre-check: any funds attached?  The DB has ON DELETE RESTRICT on
    // funds.entity_id but we want a friendly 409 with a count.
    let attached = state
        .db
        .count(
            "funds",
            &format!(
                "entity_id=eq.{entity_id}&organization_id=eq.{}",
                user.organization_id
            ),
        )
        .await?;
    if attached > 0 {
        return Err(AppError::Conflict(format!(
            "Cannot delete entity '{entity_id}': {attached} fund(s) still reference it. Reassign or remove the attached funds first."
        )));
    }

    state
        .db
        .delete(
            "entities",
            &format!(
                "id=eq.{entity_id}&organization_id=eq.{}",
                user.organization_id
            ),
        )
        .await?;

    Ok(HttpResponse::NoContent().finish())
}
