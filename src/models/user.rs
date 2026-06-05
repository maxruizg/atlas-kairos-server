use serde::{Deserialize, Serialize};

/// A platform user. The first user created at signup is the platform
/// **owner** (full admin rights). Future invited users would carry a
/// different `role` value (e.g. "member", "viewer").
///
/// `password_hash` is the PHC-formatted Argon2id hash. It is **never**
/// serialised to clients — see [`UserPublic`] for the shape returned
/// from the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    pub created_at: String,
}

/// Sanitised view of [`User`] safe to return from any endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct UserPublic {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub created_at: String,
}

impl From<&User> for UserPublic {
    fn from(u: &User) -> Self {
        Self {
            id: u.id.clone(),
            organization_id: u.organization_id.clone(),
            name: u.name.clone(),
            email: u.email.clone(),
            role: u.role.clone(),
            created_at: u.created_at.clone(),
        }
    }
}
