use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Phase 1 — expanded data model
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub short: String,
    pub nav: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Sponsor {
    pub id: String,
    pub name: String,
    pub initials: String,
    pub country: String,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Fund {
    pub id: String,
    pub sponsor_id: String,
    pub entity_id: String,
    pub name: String,
    pub vintage: u32,
    pub strategy: String,
    pub asset_class: String,
    pub geography: String,
    pub currency: String,
    pub commitment: f64,
    pub paid_in: f64,
    pub nav: f64,
    pub distributions: f64,
    pub unfunded: f64,
    pub tvpi: f64,
    pub dpi: f64,
    pub rvpi: f64,
    pub gross_irr: f64,
    pub net_irr: f64,
    pub gross_moic: f64,
    pub net_moic: f64,
    pub pct_called: f64,
    pub companies: Vec<PortfolioCompany>,
    pub nav_history: Vec<NavPoint>,
    pub cashflows: Vec<CashflowPoint>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortfolioCompany {
    pub name: String,
    pub theme: String,
    pub date: String,
    pub status: String,
    pub invested: f64,
    pub fmv: f64,
    pub moic: f64,
    pub irr: f64,
    pub own: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NavPoint {
    pub q: String,
    pub nav: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CashflowPoint {
    pub q: String,
    pub calls: f64,
    pub dist: f64,
}

// ---------------------------------------------------------------------------
// Legacy / Phase-0 structs (kept for backward compatibility until Phase 7)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Investment {
    pub id: String,
    pub name: String,
    pub sponsor: String,
    pub investment_type: String, // "VC", "PE", "Direct", "Real Assets"
    pub nav: f64,
    pub tvpi: f64,
    pub dpi: f64,
    pub rvpi: f64,
    pub irr: f64,
    pub status: String, // "green", "orange", "red"
    pub theme: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Kpi {
    pub label: String,
    pub value: String,
    pub color: String, // "white", "orange", "purple"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Theme {
    pub name: String,
    pub nav: f64,
    pub pct: u32,
    pub tvpi: f64,
    pub irr: f64,
    pub color: String, // hex color
}
