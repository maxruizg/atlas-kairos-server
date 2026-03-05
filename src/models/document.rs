use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Document {
    pub id: String,
    pub name: String,
    pub doc_type: String,
    pub fund: String,
    /// "Uploaded" | "Extracted" | "Needs Review" | "Approved" | "Posted"
    pub status: String,
    pub confidence: Option<u32>,
    pub date: String,
    pub size: String,
    pub fields: u32,
    pub extracted: u32,
    /// Links the document to a specific sponsor. `None` for newly-uploaded
    /// documents that have not yet been associated with a fund.
    pub sponsor_id: Option<String>,
    /// Links the document to a specific fund. `None` until the document is
    /// classified and associated with a fund in the portfolio.
    pub fund_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateStatusRequest {
    pub status: String,
}
