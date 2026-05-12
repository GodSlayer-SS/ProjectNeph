/// store/ — SQLite data layer (Blueprint §4: "store/ # KEEP sqlx + migrations").
///
/// This module is the authoritative home for all database access:
///   - `mod.rs`  — re-exports and module organization
///   - DB init, migrations, settings CRUD (`db.rs` content exposed here)
///
/// Existing code using `crate::db` continues to work — this module is an
/// additive re-export that gives the Blueprint-compliant `crate::store` path.

/// Re-export the full db API under the Blueprint-mandated `store` namespace.
pub use crate::db::*;
