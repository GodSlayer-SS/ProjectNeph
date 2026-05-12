/// secrets/ — OS credential store (Blueprint §4: "secrets/ # KEEP keyring").
///
/// This module is the authoritative location for API key read/write.
/// Existing code using `crate::secrets::*` continues to work — the flat
/// `secrets.rs` was converted to this directory module with no API changes.
///
/// Blueprint §10: "Windows Credential Manager via keyring — Done correctly."
/// We keep this verbatim. The `keyring` crate handles OS-level encryption.

pub use crate::secrets::*;
