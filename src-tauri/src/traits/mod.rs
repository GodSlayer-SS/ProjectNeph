/// Nephis — The 8 Stable Interfaces
///
/// Nothing in `actors/`, `tools/`, or `ui_bridge` may depend on a concrete
/// type. They import only these traits. Implementations live in `providers/`,
/// `memory/`, and `domains/` and are fully replaceable.
pub mod domain;
pub mod embedder;
pub mod llm;
pub mod memory;
pub mod planner;
pub mod stt;
pub mod tool;
pub mod tts;

pub use domain::{Capability, DomainId, ExecutionDomain};
pub use embedder::Embedder;
pub use llm::{ChatRequest, ChatResponse, LlmProvider, ProviderCapabilities, Token};
pub use memory::{MemoryHit, MemoryId, MemoryItem, MemoryStore, Query};
pub use planner::{Intent, Plan, Planner, PlannerCtx};
pub use stt::{AudioStream, SttEvent, SttProvider};
pub use tool::{PlannedAction, RiskLevel, Tool, ToolArgs, ToolManifest, ToolOutput, Validated};
pub use tts::{AudioChunk, TextStream, TtsProvider};
