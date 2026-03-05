use actix_web::{get, post, web, HttpResponse};

use crate::data::copilot::seed_suggestions;
use crate::errors::AppError;
use crate::models::copilot::{CopilotMessage, CopilotQueryRequest};
use crate::state::AppState;

/// Maximum query length accepted from clients.
///
/// Prevents abuse and ensures the mock response stays readable.
const MAX_QUERY_LEN: usize = 1_000;

/// `GET /api/v1/copilot/history`
///
/// Returns the full chat history for the current session.
#[get("/copilot/history")]
pub async fn get_history(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let history = state.copilot_history.read().await;

    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "public, max-age=60, stale-while-revalidate=300"))
        .json(&*history))
}

/// `GET /api/v1/copilot/suggestions`
///
/// Returns the list of prompt suggestions shown in the copilot UI.
///
/// Suggestions are static — they live in the data module — so no state
/// lock is needed here.
#[get("/copilot/suggestions")]
pub async fn get_suggestions() -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "public, max-age=60, stale-while-revalidate=300"))
        .json(seed_suggestions()))
}

/// `POST /api/v1/copilot/query`
///
/// Appends the user's message to the history, generates a deterministic
/// mock AI response, appends that too, and returns the AI message.
///
/// # Validation
/// - `query` must be non-empty and no longer than `MAX_QUERY_LEN`.
///
/// # Security note
/// The query is echoed back inside the AI response text.  We strip
/// leading/trailing whitespace and enforce a length cap so that a crafted
/// payload cannot produce an unbounded response body.
#[post("/copilot/query")]
pub async fn post_query(
    body: web::Json<CopilotQueryRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let query = body.query.trim().to_string();

    if query.is_empty() {
        return Err(AppError::BadRequest(
            "Query must not be empty.".to_string(),
        ));
    }

    if query.len() > MAX_QUERY_LEN {
        return Err(AppError::BadRequest(format!(
            "Query exceeds maximum length of {} characters.",
            MAX_QUERY_LEN
        )));
    }

    // Build the user message.
    let user_message = CopilotMessage {
        role: "user".to_string(),
        text: query.clone(),
        table: None,
        citations: None,
        total: None,
    };

    // Build a deterministic mock AI response that echoes the query.
    let ai_response = CopilotMessage {
        role: "ai".to_string(),
        text: format!(
            "Based on the available portfolio data, I can see that \"{}\". \
             Let me look into that for you. Currently tracking 5 investments \
             across 5 themes with a total NAV of $32.1M.",
            query
        ),
        table: None,
        citations: Some(vec!["Portfolio Ledger · Latest".to_string()]),
        total: None,
    };

    // Append both messages under the write lock.
    {
        let mut history = state.copilot_history.write().await;
        history.push(user_message);
        history.push(ai_response.clone());
    }

    log::info!("Copilot query received ({} chars).", query.len());

    Ok(HttpResponse::Ok().json(ai_response))
}
