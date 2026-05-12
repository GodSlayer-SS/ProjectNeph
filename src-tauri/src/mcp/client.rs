/// mcp/client.rs — MCP client (Blueprint §4, Phase 4).
///
/// Blueprint §4 Phase 4: "MCP client (consume external tool servers)."
///
/// The MCP (Model Context Protocol) client connects to external tool servers
/// (e.g., a Postgres MCP server, a GitHub MCP server) and exposes their tools
/// to Nephis's planner via the `Tool` trait.
///
/// Phase 4 plan:
///   1. Add `rmcp` to Cargo.toml: `cargo add rmcp`
///   2. Implement `McpClient::connect_stdio(cmd, args)` — forks subprocess
///   3. Implement `McpClient::connect_sse(url)` — HTTP+SSE transport
///   4. Route MCP tool calls through the existing `ExecutorActor` confirmation flow
///
/// Phase 1–3 stub: config structs only, all methods return "not yet enabled".

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

// ── Config ────────────────────────────────────────────────────────────────────

/// Transport type for MCP server connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpTransport {
    /// Spawn a local subprocess and communicate over stdio.
    Stdio { command: String, args: Vec<String> },
    /// Connect to a remote server over HTTP+SSE.
    Sse { url: String },
}

/// Configuration for a single MCP server connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpClientConfig {
    /// Human-readable name for this server (e.g., "github", "postgres").
    pub name: String,
    pub transport: McpTransport,
    pub auth_token: Option<String>,
}

// ── Tool descriptor (mirrors MCP spec §4.3) ──────────────────────────────────

/// A tool exposed by an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDescriptor {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

// ── Client ────────────────────────────────────────────────────────────────────

pub struct McpClient {
    pub config: McpClientConfig,
}

impl McpClient {
    pub fn new(config: McpClientConfig) -> Self {
        Self { config }
    }

    /// Connect to the MCP server and return a live client.
    ///
    /// Phase 4: implement via `rmcp` crate.
    /// Phase 1–3: always returns an error with guidance.
    pub fn connect(_config: McpClientConfig) -> Result<Self> {
        bail!(
            "MCP client is a Phase 4 feature. \
             Add 'rmcp' to Cargo.toml and implement McpClient::connect_stdio/connect_sse."
        )
    }

    /// List tools available from this MCP server.
    pub fn list_tools(&self) -> Result<Vec<McpToolDescriptor>> {
        bail!("MCP client not yet connected (Phase 4)")
    }

    /// Call a tool on this MCP server.
    pub fn call_tool(&self, _name: &str, _args: serde_json::Value) -> Result<serde_json::Value> {
        bail!("MCP client not yet connected (Phase 4)")
    }
}

// ── Registry ──────────────────────────────────────────────────────────────────

/// In-process registry of active MCP client connections.
/// Phase 4: populated from `settings.mcp_servers` JSON on startup.
pub struct McpClientRegistry {
    clients: Vec<McpClient>,
}

impl McpClientRegistry {
    pub fn new() -> Self {
        Self { clients: vec![] }
    }

    /// Add a connected client to the registry.
    pub fn register(&mut self, client: McpClient) {
        self.clients.push(client);
    }

    /// Find a tool across all registered MCP servers.
    pub fn find_tool(&self, _tool_name: &str) -> Option<&McpClient> {
        // Phase 4: iterate self.clients, call list_tools(), match by name.
        None
    }
}

impl Default for McpClientRegistry {
    fn default() -> Self {
        Self::new()
    }
}
