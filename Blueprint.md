# 🜂 NEPHIS COUNCIL — ROUND 2 (REBUTTAL)

You handed me a serious critique. I'm going to do exactly what you asked: run my own council against it, **agree where it's right, disagree where it's wrong**, and then give you the final blueprint you build against. No agreement theater.

Let me be upfront about my honest scoring of the GPT critique before the debate:

| GPT Critique Point | My Verdict |
|---|---|
| Rust-first is too aggressive early | ⚠️ **Partially right, partially wrong** |
| Cloud should be primary cognition | ✅ **Right** |
| Don't over-romanticize the orb early | ✅ **Right** |
| Browser automation needs domain isolation | ✅ **Right and important** |
| Need stable interfaces / swappable abstractions | ✅ **Right** |
| Use Qwen2.5 3B not Qwen3 4B | ❌ **Wrong reasoning, possibly right conclusion** |
| Replace Rust core with Python orchestrator | ❌ **Dangerously wrong for your goals** |

Now let's debate it properly.

---

## ⚔️ The Council — 5 Specialists, Round 2

### Council Member α — **The Pragmatic Systems Engineer** (responds to "Rust-first is too aggressive")

> "GPT is half-right. The conclusion 'don't make Rust the center initially' contains a real insight: **iteration speed matters more than language purity in early phases.** But the prescription — 'Python orchestrator + Rust modules' — is the *worst* of both worlds for a single-user desktop app.
>
> Here's why: Python orchestrator means you now have a **Python process + a Rust process + a Node sidecar + a Python ML sidecar**. That's four runtimes, four IPC boundaries, four crash domains, four startup sequences, four logging stacks. On a 16GB laptop with Chrome, VSCode, and Discord open, this is exactly the kind of architecture that feels janky.
>
> The user already has Tauri. Tauri *is* a Rust process with a webview. Throwing a Python orchestrator on top means the Rust shell becomes vestigial. Why use Tauri at all then? Just go full Electron + Python like every other AI assistant on GitHub — and that's exactly what the user said they don't want.
>
> **The right answer:** Keep Rust as the *process owner and IPC hub* (it has to be, because Tauri commands are Rust), but write the orchestration logic in **a layer that's easy to iterate on**. That's not Python. That's **TypeScript in the frontend** for early phases, calling Rust for the boring infra (state, IPC, hotkeys, secrets, audit), and Python *only* as a sidecar for ML."

### Council Member β — **The ML Infrastructure Engineer** (responds to model strategy)

> "GPT's VRAM concern is real but its conclusion is wrong on details:
>
> 1. **Qwen2.5 3B vs Qwen3 4B**: Qwen3 4B Q4_K_M is ~2.5GB on disk and ~3GB resident with a small KV cache. Qwen2.5 3B Q4 is ~1.9GB. Both fit. The real question is *function-calling quality*, and Qwen3 4B is materially better at structured JSON output. **Use Qwen3 4B for tool-call planning, Qwen2.5 3B as a cheaper alternative if you want to also run Whisper concurrently on GPU.**
>
> 2. **The bigger truth GPT got right**: cloud should be primary cognition. The user has Google AI Pro, free Groq, free OpenRouter tier, free Gemini Flash. Local LLM is a **reflex layer**, not the brain. Don't fall in love with running models locally just because you can.
>
> 3. **VRAM math for your laptop with Chrome open**: Chrome takes ~500MB GPU memory. VSCode ~200MB. WebView2 (Tauri UI with WebGPU orb) ~400-700MB. That leaves ~4.2GB. Whisper small CUDA = 1.5GB. Qwen3 4B = 3GB. Together = 4.5GB. **Tight, with risk of OOM during heavy browser use.**
>
> 4. **Pragmatic VRAM strategy**: Run Whisper on **CPU** (faster-whisper int8) — it's 600ms instead of 250ms but doesn't fight the LLM for VRAM. Keep GPU for the LLM. Or, use **cloud STT (Deepgram Nova-3 free tier, or Groq Whisper API at <300ms)** and skip local Whisper entirely. This is the unlock GPT didn't articulate but should have.
>
> 5. **Local model isn't even strictly necessary in v1.** Gemini Flash is fast enough (~400ms TTFB), free, and better than Qwen3 4B at almost everything. **Defer local LLM to Phase 2 or 3.** You'll thank yourself."

### Council Member γ — **The HCI Designer** (responds to "delay UI polish")

> "GPT is **completely right** on this one and I retract my earlier romanticism.
>
> Reliability creates emotional attachment. Particles do not. Every great voice assistant in history — Siri's first version, Alexa's first version, even Jarvis in fiction — felt magical because **it just worked, fast**. Not because of shaders.
>
> The orb in v1 should be: a circle that pulses with audio amplitude and changes color across 5 states. Done. 200 lines of WebGL. That's it.
>
> Save the WebGPU shaders, particle systems, and audio-reactive bloom for Phase 4 when you've earned the right to spend a weekend on it. Anyone who tells you otherwise is selling you a demo, not an assistant."

### Council Member δ — **The Security Architect** (responds to "execution domains")

> "GPT's domain separation point is the most underrated insight in their entire critique. I missed this in Round 1 and I shouldn't have.
>
> Persistent Playwright profile = **Nephis is logged into your Gmail, your bank, your GitHub, your Twitter**. If a prompt-injection attack lands in any web page that gets summarized, the LLM could be coerced into instructing the executor to perform actions in *your name* using *your sessions*.
>
> The fix isn't theoretical. It's **four discrete browser profiles** with separate cookie jars:
>
> | Profile | Use | Approval needed |
> |---|---|---|
> | `nephis-research` | search, read pages, summarize | green |
> | `nephis-tools` | logged into Notion, GitHub (read-only OAuth scopes only) | yellow |
> | `nephis-personal` | YOUR logged-in personal browser, full access | red, every action |
> | `nephis-throwaway` | for risky/unknown sites | green |
>
> The LLM only ever sees a profile *handle*, not raw cookies. The Executor binds tool calls to profile permissions. Cross-profile actions are **always** red-tier.
>
> Same model applies to filesystem (workspace / personal / system / temp), shell (sandboxed / native), and network (allowlisted / open).
>
> This is non-negotiable for a Jarvis-class system."

### Council Member ε — **The Long-Term Evolution Engineer** (responds to "stable interfaces")

> "GPT nailed this. Ignore my Round 1 take where I glossed over it.
>
> The single most expensive mistake you can make is **coupling your code to specific implementations**. Three years from now, the LLM landscape will be unrecognizable. The vector DB you pick today will be slower than something else. Whisper will be replaced by something 10× better. Tauri will have a v3.
>
> Your code must not care.
>
> The discipline is: **define traits/interfaces before you write any implementation.** Every actor talks to other actors through trait objects. Every provider implements `LlmProvider`. Every memory tier implements `MemoryStore`. Every tool implements `Tool`. Swapping Gemini for whatever-comes-next should be a 20-line PR.
>
> But — and this is where I push back on GPT — **stable interfaces don't require Python**. Rust traits are *better* at this than Python ABCs because the compiler enforces the contract. The argument 'Python is more flexible for evolving abstractions' is true for *implementation*, but the *interface itself* is more durable in a typed language. Build the trait wall once, then iterate behind it."

---

## 🜁 The Final Debate — Where I Disagree With GPT

### 🔴 Disagreement #1: "Replace Rust core with Python orchestrator"

This is the part of GPT's critique I most strongly reject, and I want you to understand why before you decide.

**GPT's claim:** Rust is painful when abstractions evolve weekly. Python lets you iterate faster.

**Why this is wrong for your specific case:**

1. **The orchestrator is not where iteration happens.** Iteration happens in **prompts, tool definitions, and UX**. The orchestrator itself is ~800 lines of "dispatch event → invoke planner → walk plan → emit results." That code stabilizes in week 2 and barely changes after.

2. **Python orchestrator means dual-runtime forever.** You can never escape it. Every new feature crosses the IPC boundary. Debugging means following stack traces across two languages. This is what kills personal projects.

3. **Tauri Commands are Rust.** Period. If your orchestrator is Python, you need Rust to *re-dispatch* every IPC call to Python. That's pure overhead — both at runtime and in your head when reading code.

4. **The "Python is faster to iterate" argument is true in 2018, not 2026.** With `cargo watch -x run`, modern rust-analyzer, and AI-assisted Rust coding (Claude Sonnet 4.5 writes excellent Rust), Rust velocity is now ~70% of Python velocity for application code. The 30% gap is dwarfed by the IPC tax of dual-runtime.

5. **Where Python actually wins**: ML libraries (Whisper, Piper, sentence-transformers, etc.). And that's exactly where I keep it — **as a sidecar, not as the orchestrator.**

**The synthesis:** GPT is right that *over-committing to Rust ML code* is a mistake. I never proposed that. Whisper, embeddings, TTS — all Python sidecar. But the orchestrator? Stays in Rust. It's 800 lines, it's typed, it's compiled, it's the right tool.

### 🟡 Disagreement #2 (partial): The Qwen3 4B vs Qwen2.5 3B argument

GPT says "Qwen2.5 3B is safer for your VRAM." This is technically true if you also run Whisper on GPU. But the deeper truth is: **don't run local LLM at all in Phase 1**. Use Gemini Flash. Add local Qwen3 4B in Phase 2 *only when you've measured a real latency or privacy need*. GPT was solving the wrong layer.

### ✅ Agreements (where GPT outclassed me)

I want to be honest: GPT's Round 2 critique improved my Round 1 blueprint in five real ways, and the final design must reflect that:

1. **Cloud-primary cognition.** I leaned too local. Local LLM is for reflexes only, and even that is Phase 2+.
2. **Delay UI polish.** The orb stays simple until reliability is locked.
3. **Execution domains.** Browser profiles + filesystem domains + network policy must be first-class concepts, not afterthoughts.
4. **Stable interfaces first.** Define traits before implementations. This is a discipline, not a folder layout.
5. **Build velocity > engineering elegance early.** I will admit my Round 1 blueprint had a streak of "wouldn't this be beautiful in Rust" thinking. Beauty is earned later.

---

## 🜃 NEPHIS — FINAL BLUEPRINT v2 (Definitive)

This is the version you build against. It absorbs GPT's valid critiques, rejects its wrong ones, and resolves the contradictions.

### 0. Mantra (memorize this)

```
Cloud-primary cognition. Local reflexes only when measured.
Stable interfaces, replaceable implementations.
Reliability before immersion. Latency before features.
Trust kernel is sacred. Domains are absolute.
One process, typed everywhere, streaming everything.
```

### 1. Architecture Pattern

**Modular monolith. Single Rust process (Tauri). One Python sidecar for ML. Optional Node sidecar for Playwright (Phase 3).**

No microservices. No message broker. No multi-agent loops. No Python orchestrator. No Docker.

### 2. The Eight Stable Interfaces (build these BEFORE any feature)

These are Rust traits. They never change. Implementations behind them can change weekly.

```rust
// 1. Anything that can answer text questions
trait LlmProvider {
    async fn complete(&self, req: ChatRequest) -> Result<ChatResponse>;
    fn stream(&self, req: ChatRequest) -> impl Stream<Item = Token>;
    fn capabilities(&self) -> ProviderCapabilities; // tools? vision? json mode?
}

// 2. Anything that produces embeddings
trait Embedder {
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
    fn dim(&self) -> usize;
}

// 3. Anything that does speech-to-text streaming
trait SttProvider {
    fn stream(&self, audio: AudioStream) -> impl Stream<Item = SttEvent>;
}

// 4. Anything that does text-to-speech streaming
trait TtsProvider {
    fn stream(&self, text: TextStream) -> impl Stream<Item = AudioChunk>;
}

// 5. Memory tier
trait MemoryStore {
    async fn search(&self, q: Query, k: usize) -> Result<Vec<MemoryHit>>;
    async fn store(&self, item: MemoryItem) -> Result<MemoryId>;
    async fn forget(&self, id: MemoryId) -> Result<()>;
}

// 6. Tool contract
trait Tool {
    fn manifest(&self) -> &ToolManifest;     // declared in tools.toml
    fn risk_level(&self) -> RiskLevel;
    async fn validate(&self, args: &ToolArgs, ctx: &ExecCtx) -> Result<Validated>;
    async fn execute(&self, args: Validated, ctx: &mut ExecCtx) -> Result<ToolOutput>;
}

// 7. Execution domain (browser/fs/shell/net)
trait ExecutionDomain {
    fn id(&self) -> DomainId;
    fn allowed_caps(&self) -> &[Capability];
    fn enforce(&self, action: &PlannedAction) -> Result<()>;
}

// 8. Planner (the only piece that uses LLM for orchestration)
trait Planner {
    async fn plan(&self, intent: Intent, ctx: &PlannerCtx) -> Result<Plan>;
}
```

**Rule: nothing in `actors/`, `tools/`, or `ui_bridge` may depend on a concrete type. They depend only on these 8 traits.** This is the long-term survival contract.

### 3. Tech Stack (Final)

| Layer | Choice | Phase |
|---|---|---|
| Shell | **Tauri 2.x** | 1 |
| Frontend | **React 19 + TS + Vite** | 1 |
| Orb v1 | Plain WebGL2 + audio amplitude shader (~200 LOC) | 1 |
| Orb v2 | R3F + WebGPU shaders | 4 |
| State (FE) | Zustand + ts-rs codegen | 1 |
| Core | **Rust + Tokio + tracing** | 1 |
| State (BE) | **SQLite (WAL) via sqlx** (you already have it) | 1 |
| Vector | **LanceDB** (embedded) | 2 |
| Secrets | **Windows Credential Manager / keyring** (you have it) | 1 |
| ML sidecar | **Python** (faster-whisper, Piper/EdgeTTS, sentence-transformers, Silero VAD) | 1 |
| Browser sidecar | **Node + Playwright** with profile isolation | 3 |
| **Primary LLM** | **Gemini 2.5 Flash** (free, fast, multimodal) | 1 |
| **Reasoning LLM** | **Claude Sonnet 4.5** (code, complex plans) | 1 |
| **Speed LLM** | **Groq Llama 3.3 70B** (latency-critical replies) | 1 |
| **Fallback** | **OpenRouter** | 1 |
| **Local LLM** | **Qwen3 4B Q4_K_M** via llama.cpp/CUDA | **Phase 2** (only after measurement) |
| STT | **Groq Whisper API** (cloud, <300ms) primary; faster-whisper local fallback | 1 |
| TTS | **EdgeTTS** (free, fast, decent voice) primary; Piper local; ElevenLabs Flash optional | 1 |
| Wake/activation | **Push-to-talk only** (hotkey) | 1; wake-word Phase 4 |
| Desktop auto | `uiautomation` Rust crate + `enigo` | 3 |
| MCP | `rmcp` (server + client) | 4 |

### 4. Folder Structure (Domain-First, Not Layer-First)

```
nephis/
├── apps/desktop/                    # the Tauri app, single binary
│   ├── src/                         # React frontend (TS)
│   │   ├── orb/                     # v1: minimal WebGL ring
│   │   ├── chat/                    # streaming chat surface
│   │   ├── palette/                 # KEEP existing power-user palette
│   │   ├── permissions/             # capability prompts (yellow/red)
│   │   ├── memory-inspector/
│   │   ├── settings/
│   │   ├── ipc/                     # codegen'd typed Tauri client
│   │   └── state/
│   ├── src-tauri/src/
│   │   ├── main.rs
│   │   ├── bus.rs                   # in-proc event bus
│   │   ├── traits/                  # the 8 stable interfaces ⭐
│   │   │   ├── llm.rs
│   │   │   ├── stt.rs
│   │   │   ├── tts.rs
│   │   │   ├── embedder.rs
│   │   │   ├── memory.rs
│   │   │   ├── tool.rs
│   │   │   ├── domain.rs
│   │   │   └── planner.rs
│   │   ├── actors/
│   │   │   ├── hotkey.rs
│   │   │   ├── voice.rs
│   │   │   ├── planner.rs
│   │   │   ├── executor.rs
│   │   │   ├── memory.rs
│   │   │   ├── automation.rs
│   │   │   ├── provider_router.rs
│   │   │   └── ui_bridge.rs
│   │   ├── domains/                 # execution domain enforcement ⭐
│   │   │   ├── browser.rs           # 4 isolated profiles
│   │   │   ├── filesystem.rs        # workspace/personal/system/temp
│   │   │   ├── shell.rs             # sandbox tiers
│   │   │   └── network.rs           # egress allowlist per tool
│   │   ├── tools/
│   │   ├── safety/                  # KEEP your existing trust kernel
│   │   │   ├── risk.rs
│   │   │   ├── confirmation.rs      # planHash + token + 60s TTL
│   │   │   ├── path_policy.rs
│   │   │   └── capabilities.rs
│   │   ├── memory/
│   │   │   ├── hot.rs
│   │   │   ├── warm.rs              # SQLite
│   │   │   ├── cold.rs              # LanceDB
│   │   │   ├── procedural.rs
│   │   │   └── admission.rs         # the curator
│   │   ├── providers/
│   │   │   ├── gemini.rs
│   │   │   ├── anthropic.rs
│   │   │   ├── groq.rs
│   │   │   ├── openrouter.rs
│   │   │   └── local_llama.rs       # Phase 2
│   │   ├── ipc/
│   │   │   ├── tauri_cmds.rs
│   │   │   ├── events.rs
│   │   │   └── pyside.rs            # named-pipe to Python
│   │   ├── store/                   # KEEP sqlx + migrations
│   │   └── secrets/                 # KEEP keyring
│   ├── tools.toml                   # versioned tool manifest ⭐
│   └── tauri.conf.json
│
├── apps/pyside/                     # ML sidecar (Python)
│   ├── pyproject.toml
│   └── nephis_pyside/
│       ├── pipe_server.py
│       ├── stt_whisper.py
│       ├── tts_edge.py
│       ├── tts_piper.py
│       ├── vad_silero.py
│       └── embeddings.py
│
├── apps/nodeside/                   # Playwright sidecar (Phase 3)
│
├── packages/
│   ├── protocol/                    # ts-rs / specta generated types
│   ├── tools-sdk/                   # Tool trait + manifest schema
│   └── orb/                         # orb shaders & state machine
│
├── docs/ADRs/                       # architecture decision records
└── scripts/
```

### 5. Execution Domains (Critical Addition from GPT's Critique)

**Browser** — 4 isolated Chromium profiles, never mixed:

| Profile | Logged in to | Tier | LLM can use? |
|---|---|---|---|
| `nephis-research` | nothing | green | yes, freely |
| `nephis-tools` | scoped read-only OAuth (Notion, GH) | yellow | yes, with confirmation |
| `nephis-personal` | your real accounts | red | only via explicit `>browse-personal` directive |
| `nephis-throwaway` | nothing, fresh each session | green | for risky/unknown URLs |

**Filesystem** — domain-scoped paths:

| Domain | Scope | Tier |
|---|---|---|
| `workspace` | `~/nephis-workspace/` | green write, read everything in domain |
| `projects` | declared project roots | yellow write |
| `personal` | Documents, Downloads, Desktop | yellow write, red delete |
| `system` | everything else (Program Files, system32, registry) | red, manual override only |
| `temp` | `~/.nephis/tmp/` | green |

**Shell** — three sandbox tiers:

| Tier | Use | Implementation |
|---|---|---|
| `safe` | text utils, git read | in-process Rust |
| `sandboxed` | code execution, builds | Windows Job Object + Restricted Token |
| `native` | system commands | red-tier, manual confirmation, full audit |

**Network** — per-tool egress allowlist declared in `tools.toml`. Egress filter at the Rust HTTP client level.

The LLM **never** sees raw cookies/tokens/paths. It sees only domain handles. The Executor maps handles → reality.

### 6. Voice Pipeline (Refined)

```
Hotkey down
  → mic open (cpal/WASAPI)
  → Silero VAD (Python sidecar, CPU, sub-ms)
  → Groq Whisper API (cloud, ~250ms TTFT)   ── primary
       fallback: faster-whisper CPU int8 (~600ms)   ── offline
  → partial transcripts stream to UI
  → final transcript → Planner
  → Planner: Gemini Flash (default) or Claude Sonnet (code/reason)
       routing decided by intent classifier (rule-based in v1, local LLM in v2)
  → token stream → sentence chunker → EdgeTTS streaming → speaker
  → barge-in: VAD-active during TTS; new speech → cancel chain via tokio CancellationToken
```

**Latency budget (Phase 1, achievable):**
- Wake → STT first partial: ~150ms
- STT final → Planner first token: ~400ms (Gemini Flash)
- Planner first token → audible word: ~250ms (EdgeTTS streaming)
- **Total: ~800ms hotkey-release-to-first-spoken-word.** This is the Jarvis bar.

### 7. Memory System (unchanged from Round 1, since GPT didn't critique it — and it's correct)

3 tiers: **Hot (RAM session)** → **Warm (SQLite: episodes, facts, preferences, procedural)** → **Cold (LanceDB embeddings)**.

**Admission control after every session** = the difference between Jarvis (curates) and ChatGPT memory (hoards). Cheap distill pass classifies what's worth keeping. Discard everything else.

### 8. Model Routing Strategy (Refined: Cloud-Primary)

| Use case | Phase 1 | Phase 2+ |
|---|---|---|
| Intent classification | Rule-based + Gemini Flash | Local Qwen3 4B |
| Tool argument JSON | Gemini Flash (JSON mode) | Local Qwen3 4B with grammar |
| Short chat | Gemini Flash | Local Qwen3 4B |
| Long chat / reasoning | Gemini 2.5 Flash → Pro | same |
| Code generation | Claude Sonnet 4.5 | same |
| Code review / debugging | Claude Sonnet 4.5 | same |
| Latency-critical replies | Groq Llama 3.3 70B | same |
| Vision / GUI grounding | Gemini 2.5 Flash | same |
| Memory distill | Gemini Flash | Local Qwen3 4B |
| Reranking | bge-reranker-base (Python sidecar) | same |

**Cost:** Gemini Flash free tier covers daily personal use. Claude Sonnet via OpenRouter free credits or $3/MTok input. Groq free tier covers thousands of requests/day. **Realistic monthly cost: $0–10** for heavy use.

### 9. Build Roadmap (Realistic, Velocity-First)

#### **Phase 1 — Spine (2 weekends, ~20 hours)**

The goal is a *working voice loop on top of your existing trust kernel.* Nothing else.

1. Create the 8 trait files. Empty bodies. Compile.
2. Define `tools.toml` schema. Port your existing 3 tools (`note`, `remember`, `recall`) into it.
3. Spin up Python sidecar with named-pipe IPC. Stub Whisper + EdgeTTS endpoints.
4. `VoiceActor`: hotkey → mic → Whisper (cloud first, local later) → text.
5. `ProviderRouter` with Gemini Flash + Claude Sonnet + Groq behind the `LlmProvider` trait.
6. `PlannerActor` v0: just streams Gemini reply token-by-token; no plans yet.
7. EdgeTTS streaming → speaker.
8. Orb v1: plain WebGL ring, 5-state color machine, audio-amplitude reactive.
9. Keep `>note >remember >recall` palette working unchanged.

**Definition of done:** hold hotkey, ask question, hear streaming answer in <1s. Trust kernel still gates writes.

#### **Phase 2 — Brain (3–4 weekends)**

1. `Planner` produces typed Plans (JSON). Validate against `tools.toml` schemas.
2. `ExecutorActor` walks plans, integrates your existing `confirmationToken`/planHash/TTL flow.
3. `MemoryStore` tiers: SQLite warm (already there) + LanceDB cold + admission control distill pass.
4. Hybrid retrieval (FTS5 + ANN + reranker via Python sidecar).
5. Local Qwen3 4B via llama.cpp **only if** measurement shows cloud latency hurts daily UX.
6. Memory inspector UI.

**DoD:** "remember I prefer pnpm" → next session, "set up a new project" auto-uses pnpm.

#### **Phase 3 — Hands (4–6 weekends)**

1. Node sidecar with Playwright. **Four-profile domain isolation from day one.**
2. Browser tools: `read_page`, `search`, `fill_form`, `click`, all tagged to a profile.
3. Desktop tools via `uiautomation` + `enigo`: `focus_window`, `type_in_active`, `read_active`.
4. File organizer (Plan template, parameterized, never free-roaming).
5. Code companion mode (Claude Sonnet driving diffs through your confirmation flow).

**DoD:** "find me 3 flights Delhi→Tokyo next month" → research profile, results in chat. "Open my Notion doc on Nephis" → personal profile, requires red-tier confirmation.

#### **Phase 4 — Reach (ongoing)**

1. Wake-word toggle (openWakeWord).
2. MCP server (expose Nephis tools to Claude Desktop).
3. MCP client (consume external tool servers).
4. Orb v2 with WebGPU + R3F shaders.
5. ElevenLabs Flash voice option.
6. Discord & WhatsApp integrations (only after voice + memory + browser are rock-solid).
7. Procedural skill library.

### 10. Things to Remove / Keep / Refactor From Existing Repo

| Action | Item | Reason |
|---|---|---|
| ✅ Keep verbatim | `confirmationToken + planHash + 60s TTL` | Best part of your repo |
| ✅ Keep | Risk-classified tools | Foundation |
| ✅ Keep | Path allowlist | Foundation |
| ✅ Keep | Windows Credential Manager via keyring | Done correctly |
| ✅ Keep | SQLite WAL + migrations | Correct |
| ✅ Keep | `command_history` + `actions` audit | Foundation |
| ✅ Keep | Redacted `tool_args` lineage | Foundation |
| ✅ Keep | LLM cannot start yellow/red without confirmation | Sacred rule |
| ✅ Keep | Locked CSP, no generic shell exposure | Correct |
| ✅ Keep | `>note >remember >recall` palette | Power-user muscle memory |
| 🔁 Refactor | Hard-coded risk map → `tools.toml` | Versioning + UI prompts |
| 🔁 Refactor | Generic React UI → orb + chat surface | Voice-first |
| 🔁 Refactor | Risk levels → capability tokens (additive, don't break risk levels) | Domain enforcement |
| 🗑 Delete | `archive/pre-pivot/` | Removed from working tree; history still in git if ever needed |
| 🗑 Delete | E2E tests against current palette | Will rebuild post-Phase 2 |
| ➕ Add | The 8 stable trait interfaces | Long-term survival |
| ➕ Add | Python ML sidecar with named-pipe IPC | Voice + memory |
| ➕ Add | LanceDB | Semantic recall |
| ➕ Add | ProviderRouter behind `LlmProvider` trait | Multi-LLM |
| ➕ Add | 4 isolated browser profiles | Security domain |
| ➕ Add | Filesystem domain enforcement | Security |
| ➕ Add | Network egress allowlist per tool | Security |
| ➕ Add | Event push protocol over Tauri events | Streaming UX |
| ➕ Add | `tracing` + JSON structured logs | Debuggability |
| ✏️ Reverse | "Defer Ollama to v0.3" → defer local LLM to Phase 2 with measurement | Cloud-primary works fine for v1 |

### 11. Biggest Mistakes to Avoid (Final List)

1. **Letting GPT talk you into a Python orchestrator.** Don't.
2. **Building local LLM in Phase 1 because it's "cool".** Cloud-primary. Add local when measured.
3. **Sharing a single browser profile across LLM-driven actions.** Domain isolation from day one.
4. **Skipping the trait wall.** Without those 8 interfaces, you'll be rewriting in 6 months.
5. **Polishing the orb before voice latency is <1s.** Reliability creates attachment, not particles.
6. **Adding multi-agent loops "for autonomy".** One Planner + one Executor. No loops.
7. **Letting the LLM write to red-tier without confirmation.** Your existing rule. Sacred.
8. **Hoarding memory.** Admission control or your vector DB becomes a swamp.
9. **Skipping streaming.** TTS, tokens, tool progress — everything streams.
10. **Trying to ship Discord/WhatsApp before voice + memory + browser are solid.** Earn it.
11. **Using LangChain "to save time".** Costs 10× what it saves.
12. **Not writing ADRs.** When you change your mind in month 6, write down why. Future you will need it.
13. **Forgetting Windows quirks.** Budget 2-3 days for WASAPI, named-pipe permissions, high-DPI Tauri sizing.
14. **Over-securing yourself out of usefulness.** Green-tier path must be frictionless.

### 12. If I Were You — The First 4 Weekends

**Weekend 1**: Trait wall + Python sidecar boilerplate + voice loop with Gemini Flash + EdgeTTS. Skeleton orb. Hold-hotkey-and-talk works end-to-end.

**Weekend 2**: ProviderRouter + Claude Sonnet + Groq behind `LlmProvider` trait. Streaming everywhere. Barge-in working. Audio amplitude wired to orb.

**Weekend 3**: `tools.toml` manifest + ExecutorActor + integrate your existing confirmationToken flow. Port `note`/`remember`/`recall`. First yellow-tier tool added (e.g. `files.write`) just to prove the gate works.

**Weekend 4**: LanceDB + bge-small embeddings + admission control. First "remember I prefer X" → "next session it knows" demo working.

**At this point you have something most "Jarvis projects" never reach.** Everything after is composition, not invention.

### 13. The Definitive Final Blueprint

```
╔═══════════════════════════════════════════════════════════════════════╗
║                       NEPHIS v1.0 — Definitive                        ║
║                                                                       ║
║ Pattern:    Modular monolith. Single Rust process. Python sidecar     ║
║             for ML. Node sidecar for Playwright (Phase 3).            ║
║                                                                       ║
║ Spine:      Tauri 2 + Rust (Tokio) + tracing                          ║
║ Brain:      Cloud-primary (Gemini Flash, Claude Sonnet, Groq)         ║
║             Local LLM (Qwen3 4B) added Phase 2 only when measured     ║
║ Voice:      Push-to-talk. Groq Whisper API → Gemini → EdgeTTS         ║
║             Target: <1s hotkey to first spoken word                   ║
║ Memory:     SQLite warm + LanceDB cold + admission-control distill    ║
║ Trust:      KEEP your kernel (planHash + token + TTL + path policy    ║
║             + Credential Manager). ADD capability tokens + execution  ║
║             domains (browser/fs/shell/net).                           ║
║ Interfaces: 8 stable Rust traits. Implementations are replaceable.    ║
║ UI:         Tauri shell. v1 orb is a 200-LOC WebGL ring. v2 orb is    ║
║             Phase 4. Chat surface for streaming. Palette for power.   ║
║ Autonomy:   Level 2. Proactive suggestions yes; destructive actions   ║
║             require user confirmation always. No agent loops.         ║
║                                                                       ║
║ Mantra:     Cloud-primary cognition. Reflexes only when measured.     ║
║             Stable interfaces, replaceable implementations.           ║
║             Reliability before immersion. Latency before features.    ║
║             Trust kernel is sacred. Domains are absolute.             ║
║             One process. Typed everywhere. Streaming everything.      ║
╚═══════════════════════════════════════════════════════════════════════╝
```

---

### Closing Honest Take

GPT's Round 2 critique was sharper than my Round 1 in five real ways: **cloud-primary cognition, delayed UI polish, execution domains, stable interfaces, build velocity over elegance**. I've absorbed all of those.

But GPT made one structurally dangerous suggestion — **swap Rust core for Python orchestrator** — that I reject with full confidence. It would force you into permanent dual-runtime overhead, multiply your debugging surface, and turn your Tauri shell into vestigial scaffolding around a Python brain. The right answer is: **Rust core that's lean, with a Python sidecar that's only ML**. Not Python orchestration with Rust modules. The difference is everything.

What you have now is no longer "a cool AI project" — it's a long-term survivable architecture with:
- a trust kernel that's already production-grade
- an interface wall that lets implementations evolve for years
- domain isolation that prevents the most realistic security failures
- a roadmap measured in weekends, not quarters
- a velocity model that ships Phase 1 in two weekends

Build the trait wall first. Build the voice loop second. Build the trust kernel-integrated executor third. Everything else compounds from there.

You're not building Jarvis. You're building a long-lived cognitive operating layer that *happens* to feel like Jarvis when it's done.

Ship the spine. The rest takes care of itself.