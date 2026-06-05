//! Supabase PostgREST client.
//!
//! Atlas talks to Supabase Postgres via the auto-generated PostgREST
//! API at `<SUPABASE_URL>/rest/v1`.  Auth is the `service_role` key,
//! which bypasses RLS — multi-tenancy is enforced in app code by
//! including `organization_id=eq.<uuid>` on every read/write.
//!
//! Each public method maps to the equivalent PostgREST verb:
//! - `select`         → GET    /rest/v1/<table>?<filter>
//! - `select_one`     → GET    + single-row (or 404)
//! - `insert`         → POST   /rest/v1/<table>  (Prefer: return=representation)
//! - `update`         → PATCH  /rest/v1/<table>?<filter>
//! - `delete`         → DELETE /rest/v1/<table>?<filter>
//!
//! Errors are mapped onto [`crate::errors::AppError`] so handlers can
//! `?` them straight through.

use reqwest::{header, Client, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use std::env;

use crate::errors::AppError;

/// Lazy-built HTTP client + Supabase project URL + service-role key.
#[derive(Clone)]
pub struct Db {
    client: Client,
    base_url: String,    // "<SUPABASE_URL>/rest/v1"
    service_key: String, // SUPABASE_SERVICE_ROLE_KEY
}

impl Db {
    /// Build a [`Db`] from `SUPABASE_URL` and `SUPABASE_SERVICE_ROLE_KEY`.
    /// Returns an explanatory error if either is missing — the backend
    /// fails fast at startup so the operator sees the misconfiguration
    /// immediately.
    pub fn from_env() -> Result<Self, String> {
        let supabase_url = env::var("SUPABASE_URL")
            .map_err(|_| "SUPABASE_URL is not set in the environment".to_string())?;
        let service_key = env::var("SUPABASE_SERVICE_ROLE_KEY").map_err(|_| {
            "SUPABASE_SERVICE_ROLE_KEY is not set in the environment".to_string()
        })?;

        let base_url = format!("{}/rest/v1", supabase_url.trim_end_matches('/'));

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

        Ok(Self {
            client,
            base_url,
            service_key,
        })
    }

    fn endpoint(&self, table: &str) -> String {
        format!("{}/{}", self.base_url, table)
    }

    fn auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        req.header("apikey", &self.service_key)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.service_key))
    }

    /// SELECT one row, returning 404 when the filter matches nothing.
    pub async fn select_one<T: DeserializeOwned>(
        &self,
        table: &str,
        filter: &str,
    ) -> Result<T, AppError> {
        let url = format!("{}?{}&limit=1", self.endpoint(table), filter);
        let res = self
            .auth(self.client.get(&url))
            .header(header::ACCEPT, "application/vnd.pgrst.object+json")
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("PostgREST request failed: {e}")))?;

        if res.status() == StatusCode::NOT_FOUND {
            return Err(AppError::NotFound(format!(
                "{table} row not found for filter '{filter}'"
            )));
        }
        if !res.status().is_success() {
            return Err(map_http_error(table, res).await);
        }
        res.json::<T>()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse {table} row: {e}")))
    }

    /// SELECT many rows; empty filter (`""`) returns everything.
    pub async fn select<T: DeserializeOwned>(
        &self,
        table: &str,
        filter: &str,
    ) -> Result<Vec<T>, AppError> {
        let url = if filter.is_empty() {
            self.endpoint(table)
        } else {
            format!("{}?{}", self.endpoint(table), filter)
        };
        let res = self
            .auth(self.client.get(&url))
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("PostgREST request failed: {e}")))?;

        if !res.status().is_success() {
            return Err(map_http_error(table, res).await);
        }
        res.json::<Vec<T>>()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse {table} list: {e}")))
    }

    /// INSERT a single row, returning the inserted row.
    pub async fn insert<B: Serialize, T: DeserializeOwned>(
        &self,
        table: &str,
        body: &B,
    ) -> Result<T, AppError> {
        let res = self
            .auth(self.client.post(&self.endpoint(table)))
            .header("Prefer", "return=representation")
            .header(header::ACCEPT, "application/vnd.pgrst.object+json")
            .json(body)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("PostgREST insert failed: {e}")))?;

        if !res.status().is_success() {
            return Err(map_http_error(table, res).await);
        }
        res.json::<T>()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse inserted {table}: {e}")))
    }

    /// UPDATE matching rows; returns the first updated row (typically scoped to one).
    pub async fn update<B: Serialize, T: DeserializeOwned>(
        &self,
        table: &str,
        filter: &str,
        body: &B,
    ) -> Result<T, AppError> {
        let url = format!("{}?{}", self.endpoint(table), filter);
        let res = self
            .auth(self.client.patch(&url))
            .header("Prefer", "return=representation")
            .header(header::ACCEPT, "application/vnd.pgrst.object+json")
            .json(body)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("PostgREST update failed: {e}")))?;

        if res.status() == StatusCode::NOT_FOUND {
            return Err(AppError::NotFound(format!(
                "{table} row not found for filter '{filter}'"
            )));
        }
        if !res.status().is_success() {
            return Err(map_http_error(table, res).await);
        }
        res.json::<T>()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse updated {table}: {e}")))
    }

    /// DELETE matching rows. Returns 204 on success.
    pub async fn delete(&self, table: &str, filter: &str) -> Result<(), AppError> {
        let url = format!("{}?{}", self.endpoint(table), filter);
        let res = self
            .auth(self.client.delete(&url))
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("PostgREST delete failed: {e}")))?;

        if !res.status().is_success() {
            return Err(map_http_error(table, res).await);
        }
        Ok(())
    }

    /// COUNT rows matching a filter using PostgREST's exact-count header.
    pub async fn count(&self, table: &str, filter: &str) -> Result<u64, AppError> {
        let url = if filter.is_empty() {
            self.endpoint(table)
        } else {
            format!("{}?{}", self.endpoint(table), filter)
        };
        let res = self
            .auth(self.client.head(&url))
            .header("Prefer", "count=exact")
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("PostgREST count failed: {e}")))?;

        if !res.status().is_success() {
            return Err(map_http_error(table, res).await);
        }

        // PostgREST returns the total in `Content-Range: 0-N/total` (or `*/total`).
        let total = res
            .headers()
            .get("content-range")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split('/').nth(1))
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        Ok(total)
    }
}

async fn map_http_error(table: &str, res: reqwest::Response) -> AppError {
    let status = res.status();
    let body = res.text().await.unwrap_or_default();
    let msg = parse_pgrst_message(&body).unwrap_or_else(|| body.clone());

    match status {
        StatusCode::CONFLICT => AppError::Conflict(msg),
        StatusCode::NOT_FOUND => AppError::NotFound(msg),
        StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY => AppError::BadRequest(msg),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
            log::error!(
                "Supabase auth failed for {table}: {status} {body}. \
                 Check SUPABASE_SERVICE_ROLE_KEY in .env."
            );
            AppError::Internal("Storage auth failed".into())
        }
        _ => AppError::Internal(format!(
            "Unexpected PostgREST status {status} for {table}: {msg}"
        )),
    }
}

/// PostgREST returns errors as `{ "code": "...", "message": "...", ...}`.
/// Pull the message field if we can; otherwise fall back to the raw body.
fn parse_pgrst_message(body: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    v.get("message")
        .and_then(|m| m.as_str())
        .map(|s| s.to_string())
}
