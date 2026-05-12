/// Domain enforcement module — Blueprint §5.
///
/// All 4 Blueprint domain types implemented:
///   filesystem — workspace/projects/personal/system/temp
///   network    — per-tool egress allowlist
///   browser    — 4 isolated Chromium profiles
///   shell      — safe/sandboxed/native tiers
pub mod filesystem;
pub mod network;
pub mod browser;
pub mod shell;
