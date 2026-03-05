use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReviewDocument {
    pub id: String,
    pub name: String,
    pub fund: String,
    pub doc_type: String,
    pub confidence: u32,
    pub fields: Vec<ReviewField>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReviewField {
    pub id: String,
    pub key: String,
    pub value: String,
    pub confidence: u32,
    pub page: u32,
    /// `None` = pending, `Some(true)` = approved, `Some(false)` = rejected
    pub approved: Option<bool>,
    pub flagged: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FieldActionRequest {
    pub approved: bool,
}
