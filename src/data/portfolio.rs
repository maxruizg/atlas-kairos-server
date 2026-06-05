use std::sync::LazyLock;

use crate::models::portfolio::{Investment, Kpi, Theme};

// ---------------------------------------------------------------------------
// Legacy / Phase-0 statics — kept as empty arrays so legacy handlers
// (`/portfolio/investments`, `/portfolio/kpis`, `/portfolio/themes`) continue
// to respond with a valid empty list rather than 500.
//
// All Phase-1 collections (entities, sponsors, funds, organizations,
// users, documents) now live in Supabase Postgres — see
// `backend/src/db.rs` and the per-handler queries.
// ---------------------------------------------------------------------------

pub static INVESTMENTS: LazyLock<Vec<Investment>> = LazyLock::new(Vec::new);

pub static KPIS: LazyLock<Vec<Kpi>> = LazyLock::new(Vec::new);

pub static THEMES: LazyLock<Vec<Theme>> = LazyLock::new(Vec::new);
