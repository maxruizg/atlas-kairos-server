use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Phase 1 — expanded data model
// ---------------------------------------------------------------------------

/// A single beneficial owner / control person attached to an entity.
///
/// Used for KYC's "ownership / control" structure. Multiple owners may
/// be attached to one entity. `ownership_pct` is in percent (0–100).
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct BeneficialOwner {
    pub name: String,
    #[serde(default)]
    pub ownership_pct: f64,
    #[serde(default)]
    pub role: Option<String>,
}

/// A legal entity within the family-office portfolio.
///
/// Phase-1 fields (`id`, `name`, `short`, `nav`) remain required; every
/// other field is optional / defaulted so the platform stays usable even
/// with minimal KYC data, but all standard family-office KYC fields are
/// available when the user wants to fill them in via the AddEntity drawer.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Entity {
    // ----- Phase 1: required core -----
    pub id: String,
    /// Tenant scoping — set by the backend on insert; never trusted from
    /// the client.  Defaulted so handlers can deserialise client payloads
    /// that omit it.
    #[serde(default)]
    pub organization_id: String,
    pub name: String,
    pub short: String,
    pub nav: f64,

    // ----- Legal identity -----
    #[serde(default)]
    pub entity_type: Option<String>,
    /// Country of formation (ISO name or code).
    #[serde(default)]
    pub jurisdiction_country: Option<String>,
    /// State / province of formation when applicable.
    #[serde(default)]
    pub jurisdiction_state: Option<String>,
    /// ISO-8601 date string (YYYY-MM-DD).
    #[serde(default)]
    pub formation_date: Option<String>,
    /// EIN / TIN / equivalent.
    #[serde(default)]
    pub tax_id: Option<String>,
    #[serde(default)]
    pub registration_number: Option<String>,
    /// One of: "Active", "Dormant", "Dissolved".
    #[serde(default)]
    pub status: Option<String>,

    // ----- Registered address -----
    #[serde(default)]
    pub address_street: Option<String>,
    #[serde(default)]
    pub address_city: Option<String>,
    #[serde(default)]
    pub address_state: Option<String>,
    #[serde(default)]
    pub address_postal_code: Option<String>,
    #[serde(default)]
    pub address_country: Option<String>,

    // ----- Primary contact -----
    #[serde(default)]
    pub contact_name: Option<String>,
    #[serde(default)]
    pub contact_title: Option<String>,
    #[serde(default)]
    pub contact_email: Option<String>,
    #[serde(default)]
    pub contact_phone: Option<String>,
    #[serde(default)]
    pub website: Option<String>,

    // ----- Tax & compliance -----
    /// Tax classification: "Pass-through", "Corporation", "Foreign",
    /// "Tax-exempt", "Other", or unset.
    #[serde(default)]
    pub tax_classification: Option<String>,
    /// FATCA status: "US Person", "Active NFFE", "Passive NFFE", "FFI",
    /// "N/A", or unset.
    #[serde(default)]
    pub fatca_status: Option<String>,
    /// ISO-8601 date the entity's KYC was last verified.
    #[serde(default)]
    pub kyc_verified_date: Option<String>,
    #[serde(default)]
    pub kyc_verified_by: Option<String>,
    /// One of: "Low", "Medium", "High".
    #[serde(default)]
    pub risk_rating: Option<String>,

    // ----- Beneficial ownership -----
    #[serde(default)]
    pub beneficial_owners: Vec<BeneficialOwner>,

    // ----- Financial -----
    /// ISO 4217 currency code (USD, EUR, MXN, etc.).
    #[serde(default)]
    pub base_currency: Option<String>,
    /// Free-form narrative (capped at the handler).
    #[serde(default)]
    pub source_of_wealth: Option<String>,

    // ----- Notes -----
    #[serde(default)]
    pub notes: Option<String>,
}

/// Tenant-level branding & onboarding state.
///
/// `id` is the row's UUID in `public.organizations`.  `name` shows up
/// in the TopBar and on every screen as the family / company brand.
/// `onboarded` gates access to the main app — until set to true, the
/// user is redirected to the onboarding wizard.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct OrganizationSettings {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub onboarded: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Sponsor {
    pub id: String,
    #[serde(default)]
    pub organization_id: String,
    pub name: String,
    pub initials: String,
    pub country: String,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Fund {
    pub id: String,
    #[serde(default)]
    pub organization_id: String,
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
    pub latest_report_q: String,
    pub report_received: bool,
    pub transactions: Vec<Transaction>,
    pub companies: Vec<PortfolioCompany>,
    pub nav_history: Vec<NavPoint>,
    pub cashflows: Vec<CashflowPoint>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub date: String,
    pub tx_type: String,
    pub amount: f64,
    pub note: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortfolioCompany {
    pub name: String,
    pub theme: String,
    pub stage: Option<String>,
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
