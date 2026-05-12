/// hotkey.rs — Hotkey parsing utilities.
///
/// Blueprint §4: this functionality lives at `actors/hotkey.rs`.
/// This file re-exports from `actors::hotkey` so existing callers
/// (`lib.rs` using `hotkey::parse_hotkey(...)`) continue to compile
/// without path changes.

pub use crate::actors::hotkey::*;
