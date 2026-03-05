use crate::models::copilot::{CopilotMessage, CopilotSuggestion};

pub fn seed_history() -> Vec<CopilotMessage> {
    vec![
        CopilotMessage {
            role: "user".to_string(),
            text: "What is our total unfunded commitment by asset class?".to_string(),
            table: None,
            citations: None,
            total: None,
        },
        CopilotMessage {
            role: "ai".to_string(),
            text: "Based on current ledger data (as of November 15, 2025), total unfunded commitments are:".to_string(),
            table: Some(vec![
                vec![
                    "Asset Class".to_string(),
                    "Unfunded".to_string(),
                    "% of Total".to_string(),
                ],
                vec![
                    "Real Assets".to_string(),
                    "$3,200,000".to_string(),
                    "44%".to_string(),
                ],
                vec![
                    "Venture Capital".to_string(),
                    "$2,600,000".to_string(),
                    "36%".to_string(),
                ],
                vec![
                    "Private Equity".to_string(),
                    "$850,000".to_string(),
                    "12%".to_string(),
                ],
                vec![
                    "Direct / Co-invest".to_string(),
                    "$550,000".to_string(),
                    "8%".to_string(),
                ],
            ]),
            citations: Some(vec![
                "Apex Q3 Statement \u{00B7} p.3".to_string(),
                "LatAm Infra Capital Call \u{00B7} p.1".to_string(),
                "Meridian Statement \u{00B7} p.2".to_string(),
            ]),
            total: Some("$7,200,000".to_string()),
        },
        CopilotMessage {
            role: "user".to_string(),
            text: "Which investment has the highest IRR in our portfolio?".to_string(),
            table: None,
            citations: None,
            total: None,
        },
        CopilotMessage {
            role: "ai".to_string(),
            text: "TechBridge Direct Co-invest leads with a gross IRR of 31.7% (XIRR, paid-in weighted). It is an unrealized position with TVPI of 2.10x and DPI of 0.00x.".to_string(),
            table: None,
            citations: Some(vec![
                "TechBridge Distribution Notice \u{00B7} Ledger Entry #2041".to_string(),
            ]),
            total: None,
        },
    ]
}

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
