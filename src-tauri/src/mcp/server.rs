/// mcp/server.rs — MCP server (Blueprint §4, Phase 4).
///
/// Blueprint §4 Phase 4: "MCP server (expose Nephis tools to Claude Desktop)."
///
/// The MCP server exposes Nephis's own tools via the Model Context Protocol
/// so external AI clients (Claude Desktop, Cursor, etc.) can call them.
///
/// Phase 4 plan:
///   1. Add `rmcp` to Cargo.toml
///   2. Implement `McpServer::start_stdio()` — listens on stdin/stdout
///   3. Map each tool from `tools.toml` to an MCP tool descriptor
///   4. Route incoming MCP calls through `ExecutorActor` with full trust kernel
///
/// Phase 1–3 stub: registration structs only.

use serde::{Deserialize, Serialize};

// ── Server registration ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpServerTransport {
    /// Expose via stdio (compatible with Claude Desktop `mcp` config entry).
    Stdio,
    /// Expose via HTTP+SSE on a local port.
    Sse { port: u16 },
}

/// Configuration for the Nephis MCP server endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub version: String,
    pub transport: McpServerTransport,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            name: "nephis".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            transport: McpServerTransport::Stdio,
        }
    }
}

// ── Built-in registrations ────────────────────────────────────────────────────

/// Returns the default server registrations that Nephis will advertise.
/// Phase 4: these will be backed by real `rmcp` transport implementations.
pub fn built_in_registrations() -> Vec<McpServerConfig> {
    vec![
        McpServerConfig {
            name: "nephis-local".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            transport: McpServerTransport::Stdio,
        },
    ]
}

// ── Server ────────────────────────────────────────────────────────────────────

pub struct McpServer {
    config: McpServerConfig,
}

impl McpServer {
    pub fn new(config: McpServerConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &McpServerConfig {
        &self.config
    }

    /// Start the MCP server. Phase 4 implementation required.
    ///
    /// Phase 4 checklist:
    ///   1. `cargo add rmcp`
    ///   2. Use `rmcp::serve_server(handler, transport).await`
    ///   3. Handler maps incoming `tools/call` → `ExecutorActor::execute_plan_step`
    ///   4. Enforce trust kernel: all MCP calls are yellow-tier minimum
    pub fn start(&self) -> anyhow::Result<()> {
        anyhow::bail!(
            "MCP server not yet implemented (Phase 4). \
             Add rmcp to Cargo.toml and implement McpServer::start()."
        )
    }
}
