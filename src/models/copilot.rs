use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CopilotMessage {
    pub role: String, // "user" or "ai"
    pub text: String,
    pub table: Option<Vec<Vec<String>>>,
    pub citations: Option<Vec<String>>,
    pub total: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CopilotQueryRequest {
    pub query: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CopilotSuggestion {
    pub text: String,
}
