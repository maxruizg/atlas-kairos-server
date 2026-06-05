use serde::{Deserialize, Serialize};

use crate::models::user::UserPublic;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub confirm_password: String,
    /// Family / company / firm name. The first user signing up bootstraps
    /// the tenant — this string becomes the organization's display name.
    #[serde(default)]
    pub organization_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub token: String,
    pub user: UserPublic,
}
