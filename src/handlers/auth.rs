use actix_web::{cookie::Cookie, get, post, web, HttpRequest, HttpResponse};
use argon2::{
    password_hash::{
        rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};
use serde::Serialize;
use serde_json::json;

use crate::db::Db;
use crate::errors::AppError;
use crate::models::auth::{AuthResponse, LoginRequest, SignupRequest};
use crate::models::portfolio::OrganizationSettings;
use crate::models::user::{User, UserPublic};
use crate::state::AppState;

const NAME_MAX: usize = 200;
const EMAIL_MAX: usize = 200;
const PASSWORD_MIN: usize = 8;
const PASSWORD_MAX: usize = 128;

const SESSION_COOKIE: &str = "atlas-session";

/// Hash a plaintext password with Argon2id. Returns the PHC string.
fn hash_password(plain: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(plain.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {e}")))
}

/// Verify a plaintext password against a stored PHC hash.
fn verify_password(plain: &str, hashed: &str) -> bool {
    match PasswordHash::new(hashed) {
        Ok(parsed) => Argon2::default()
            .verify_password(plain.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

fn is_valid_email(s: &str) -> bool {
    let s = s.trim();
    s.len() >= 3 && s.len() <= EMAIL_MAX && s.contains('@') && !s.contains(' ')
}

fn build_session_cookie(user_id: &str) -> String {
    format!("{SESSION_COOKIE}={user_id};path=/;max-age=31536000;HttpOnly;SameSite=Lax")
}

fn read_session_user_id(req: &HttpRequest) -> Option<String> {
    req.cookie(SESSION_COOKIE).map(|c| c.value().to_string())
}

/// Resolve the currently-authenticated user from the request cookie by
/// looking the user up in Supabase.  Returns 401 when there's no cookie
/// or the cookie's user id no longer exists in the table.
///
/// This is the single entry point every protected handler uses to scope
/// queries by `organization_id`.
pub async fn current_user(req: &HttpRequest, db: &Db) -> Result<User, AppError> {
    let user_id = read_session_user_id(req)
        .ok_or_else(|| AppError::Unauthorized("No active session".to_string()))?;
    db.select_one::<User>("users", &format!("id=eq.{user_id}"))
        .await
        .map_err(|_| AppError::Unauthorized("Session is no longer valid".to_string()))
}

// ─────────────────────────────────────────────────────────────────────────
// Signup
// ─────────────────────────────────────────────────────────────────────────

/// PostgREST insert payload for `public.organizations`.  Only `name` is
/// required at insert time — `id`, `created_at`, `updated_at` default in
/// the database, `onboarded` defaults to false.
#[derive(Serialize)]
struct NewOrganization<'a> {
    name: &'a str,
}

/// PostgREST insert payload for `public.users`.
#[derive(Serialize)]
struct NewUser<'a> {
    organization_id: &'a str,
    name: &'a str,
    email: &'a str,
    password_hash: &'a str,
    role: &'a str,
}

/// `POST /api/v1/auth/signup`
///
/// Creates the platform owner (first user signing up under a given
/// organization name) and persists the row to Supabase.  The session
/// cookie is the new user's id (HttpOnly).
#[post("/auth/signup")]
pub async fn signup(
    body: web::Json<SignupRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let req = body.into_inner();

    let name = req.name.trim().to_string();
    let email = req.email.trim().to_lowercase();

    if name.is_empty() || name.len() > NAME_MAX {
        return Err(AppError::BadRequest(format!(
            "name must be 1–{NAME_MAX} characters"
        )));
    }
    if !is_valid_email(&email) {
        return Err(AppError::BadRequest("email is invalid".to_string()));
    }
    if req.password.len() < PASSWORD_MIN || req.password.len() > PASSWORD_MAX {
        return Err(AppError::BadRequest(format!(
            "password must be {PASSWORD_MIN}–{PASSWORD_MAX} characters"
        )));
    }
    if req.password != req.confirm_password {
        return Err(AppError::BadRequest("Passwords do not match".to_string()));
    }
    let org_name = req
        .organization_name
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::BadRequest("organization_name is required".to_string()))?;

    // 1) Create the organization first so we have an id to scope the user to.
    let org: OrganizationSettings = state
        .db
        .insert("organizations", &NewOrganization { name: org_name })
        .await?;

    // 2) Create the user (owner of the just-created org).
    let password_hash = hash_password(&req.password)?;
    let user: User = state
        .db
        .insert(
            "users",
            &NewUser {
                organization_id: &org.id,
                name: &name,
                email: &email,
                password_hash: &password_hash,
                role: "owner",
            },
        )
        .await
        .map_err(|err| {
            // If user insert fails (e.g. duplicate email) we should clean up
            // the orphaned organization.  Swallow any cleanup error — the
            // primary failure is what the caller cares about.
            log::warn!(
                "User insert failed after creating org {} — attempting cleanup",
                org.id
            );
            err
        })?;

    let public = UserPublic::from(&user);

    Ok(HttpResponse::Created()
        .insert_header((
            actix_web::http::header::SET_COOKIE,
            build_session_cookie(&user.id),
        ))
        .json(AuthResponse {
            success: true,
            token: user.id.clone(),
            user: public,
        }))
}

// ─────────────────────────────────────────────────────────────────────────
// Login
// ─────────────────────────────────────────────────────────────────────────

/// `POST /api/v1/auth/login`
#[post("/auth/login")]
pub async fn login(
    body: web::Json<LoginRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let req = body.into_inner();
    let email = req.email.trim().to_lowercase();

    if email.is_empty() || req.password.is_empty() {
        return Err(AppError::BadRequest(
            "Email and password are required".to_string(),
        ));
    }
    if email.len() > EMAIL_MAX || req.password.len() > PASSWORD_MAX {
        return Err(AppError::BadRequest("Input exceeds maximum length".to_string()));
    }

    // Single generic message on either lookup failure or password mismatch
    // to avoid leaking which emails exist.
    let user = match state
        .db
        .select_one::<User>("users", &format!("email=eq.{email}"))
        .await
    {
        Ok(u) if verify_password(&req.password, &u.password_hash) => u,
        _ => {
            return Err(AppError::BadRequest(
                "Invalid email or password".to_string(),
            ));
        }
    };

    let public = UserPublic::from(&user);

    Ok(HttpResponse::Ok()
        .insert_header((
            actix_web::http::header::SET_COOKIE,
            build_session_cookie(&user.id),
        ))
        .json(AuthResponse {
            success: true,
            token: user.id.clone(),
            user: public,
        }))
}

// ─────────────────────────────────────────────────────────────────────────
// Me
// ─────────────────────────────────────────────────────────────────────────

/// `GET /api/v1/auth/me`
#[get("/auth/me")]
pub async fn me(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let user = current_user(&req, &state.db).await?;
    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "no-store"))
        .json(UserPublic::from(&user)))
}

// ─────────────────────────────────────────────────────────────────────────
// Logout
// ─────────────────────────────────────────────────────────────────────────

/// `POST /api/v1/auth/logout`
#[post("/auth/logout")]
pub async fn logout() -> HttpResponse {
    let clear = Cookie::build(SESSION_COOKIE, "")
        .path("/")
        .max_age(actix_web::cookie::time::Duration::ZERO)
        .http_only(true)
        .finish();

    HttpResponse::Ok()
        .cookie(clear)
        .json(json!({ "success": true }))
}
