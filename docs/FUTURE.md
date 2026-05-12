# Deferred product ideas (post v0.1)

Items intentionally **not** shipped in the current trust scope; tracked here so the codebase stays small and reviewable.

- **Local LLM (e.g. Qwen via llama.cpp)** — follow **`Blueprint.md`**: cloud-primary; add local only after measured latency/privacy need, behind the same `LlmProvider` trait.
- **`>overwritefile` command** — high-impact file mutation; prefer editor-based workflows or a gated, strongly confirmed tool later.
- **`workflows` table / automation** — persistence and UX for multi-step flows once core palette reliability is proven.
- **Dynamic plugin loading** — no runtime extension surface in v0.1; any “plugin” work should start as in-process, versioned modules with a design doc.

When picking one of these up, add a short ADR or GitHub issue linking requirements, threat model, and rollback.
