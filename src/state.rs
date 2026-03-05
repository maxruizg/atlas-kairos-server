use tokio::sync::RwLock;

use crate::data::{copilot, documents, review};
use crate::models::{
    copilot::CopilotMessage,
    document::Document,
    review::ReviewDocument,
};

/// Shared mutable application state.
///
/// Wrapped in `tokio::sync::RwLock` to allow concurrent read access while
/// serialising writes — appropriate for a read-heavy API serving mostly
/// GET requests.  Placed in an `actix_web::web::Data<AppState>` so Actix
/// clones the `Arc` (not the locks) for each handler invocation.
pub struct AppState {
    pub documents: RwLock<Vec<Document>>,
    pub review: RwLock<ReviewDocument>,
    pub copilot_history: RwLock<Vec<CopilotMessage>>,
}

impl AppState {
    /// Construct [`AppState`] from the seed data.
    pub fn new() -> Self {
        Self {
            documents: RwLock::new(documents::seed_documents()),
            review: RwLock::new(review::seed_review()),
            copilot_history: RwLock::new(copilot::seed_history()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
