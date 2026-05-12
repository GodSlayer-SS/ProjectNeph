# ADR-001: LlmProvider — Synchronous Callbacks vs async_trait

**Status:** Accepted  
**Date:** 2026-05-11  
**Context:** Blueprint §2 specifies `async fn complete()` returning `Result<ChatResponse>` and `fn stream()` returning `impl Stream<Item = Token>`.

## Decision

The `LlmProvider` trait uses synchronous signatures with an `FnMut(&str)` callback for streaming, rather than `async fn` methods or `impl Stream<Item = Token>`.

```rust
pub trait LlmProvider: Send + Sync {
    fn complete(&self, req: &ChatRequest, api_key: &str) -> Result<ChatResponse>;
    fn complete_stream(
        &self, req: &ChatRequest, api_key: &str,
        on_token: &mut dyn FnMut(&str),
    ) -> Result<ChatResponse>;
}
```

## Rationale

1. **async_trait complexity**: Rust's async-in-trait story, while improving with AFIT (async fn in traits, stable Rust 1.75+), still generates complex associated futures when combined with `dyn Trait`. Tauri commands call `LlmProvider` from blocking thread contexts (via `std::thread::spawn`), making the async overhead unnecessary.

2. **Tauri command model**: Tauri v2 Rust commands are synchronous by default. The `#[tauri::command]` macro handles the async-to-sync bridge. All provider calls happen inside `std::thread::spawn`, which is already off the main thread. An `FnMut` callback is sufficient to deliver tokens to the Tauri event bus.

3. **reqwest blocking**: All HTTP calls use `reqwest`'s blocking client. Mixing `reqwest::blocking` with `tokio::async_trait` creates a nested runtime problem on Windows.

## Consequences

- Implementations use `reqwest::blocking::Client` — simple, no runtime bridging needed.
- Streaming is caller-pull via callback, not push via `Stream`. Callers must not block inside the callback.
- When Rust's AFIT stabilizes fully for `dyn Trait` dispatch (expected ~2026), this can be revisited as a zero-breaking-change migration behind the trait wall.

## Future State

If the system moves to a tokio-native async model (e.g., Phase 4 with MCP streaming), replace with:
```rust
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, req: &ChatRequest) -> Result<ChatResponse>;
    fn stream(&self, req: ChatRequest) -> impl Stream<Item = Token> + Send;
}
```
This is a 20-line PR per provider, enabled by the trait wall.
