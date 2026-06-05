use tokio::sync::RwLock;

use crate::data::{copilot, review};
use crate::db::Db;
use crate::models::{copilot::CopilotMessage, review::ReviewDocument};

/// Shared mutable application state.
///
/// `db` is the Supabase PostgREST client — every table that has a
/// migration in `supabase/migrations/` is read/written through it.
///
/// Two tables don't have migrations yet (review, copilot history) so
/// they remain in-memory; they will move to Supabase in a follow-up
/// commit alongside their SQL migrations.
pub struct AppState {
    pub db: Db,
    /// Single-document review state. No `public.review_documents` table yet.
    pub review: RwLock<ReviewDocument>,
    /// Chat history with the AI copilot. No `public.copilot_messages` table yet.
    pub copilot_history: RwLock<Vec<CopilotMessage>>,
}

impl AppState {
    /// Construct [`AppState`] from a live [`Db`] handle and the in-memory
    /// seeds for the two tables that haven't been migrated yet.
    pub fn new(db: Db) -> Self {
        Self {
            db,
            review: RwLock::new(review::seed_review()),
            copilot_history: RwLock::new(copilot::seed_history()),
        }
    }
}
