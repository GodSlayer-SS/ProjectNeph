/// Actor module — Blueprint §4.
///
/// Phase 1 actors (functional): cancel, voice, planner, executor, hotkey.
/// `automation` holds Phase 3 desktop helpers (`desktop_*`) used by `state/runner`.
/// memory, provider_router, ui_bridge: wiring shells / routers as the spine grows.
pub mod cancel;
pub mod voice;
pub mod planner;
pub mod executor;
pub mod hotkey;
pub mod memory;
pub mod automation;
pub mod provider_router;
pub mod ui_bridge;
