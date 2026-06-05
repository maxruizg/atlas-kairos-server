use crate::models::review::ReviewDocument;

/// Seed an "empty" review document.
///
/// `ReviewDocument` is a single struct (not a `Vec`), so to represent
/// "no review pending" we initialise it with empty/zero values and a
/// sentinel id (`""`). The `GET /review/{id}` handler 404s for any id
/// that doesn't match — including `""` — which is what we want until
/// a real document is sent through the review pipeline.
pub fn seed_review() -> ReviewDocument {
    ReviewDocument {
        id: String::new(),
        name: String::new(),
        fund: String::new(),
        doc_type: String::new(),
        confidence: 0,
        fields: Vec::new(),
    }
}
