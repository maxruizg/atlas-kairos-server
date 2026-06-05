use crate::models::copilot::{CopilotMessage, CopilotSuggestion};

/// Seed an empty copilot message history. Messages accumulate in the
/// `AppState::copilot_history` lock as users submit queries.
pub fn seed_history() -> Vec<CopilotMessage> {
    Vec::new()
}

/// Suggestions remain hard-coded — they are static UI hints, not user data.
pub fn seed_suggestions() -> Vec<CopilotSuggestion> {
    vec![
        CopilotSuggestion {
            text: "Total unfunded by asset class?".to_string(),
        },
        CopilotSuggestion {
            text: "Best performing fund by gross IRR?".to_string(),
        },
        CopilotSuggestion {
            text: "Show TVPI breakdown by sponsor".to_string(),
        },
        CopilotSuggestion {
            text: "What capital calls are due this quarter?".to_string(),
        },
        CopilotSuggestion {
            text: "Compare net IRR across all funds".to_string(),
        },
    ]
}
