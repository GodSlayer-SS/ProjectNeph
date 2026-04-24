use std::{fs::OpenOptions, io::Write, path::Path};

use anyhow::Result;

use crate::redaction::redact_secrets;

pub fn write_crash_log(log_dir: &Path, message: &str) -> Result<()> {
    std::fs::create_dir_all(log_dir)?;
    let log_path = log_dir.join("neph-crash.log");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;
    let safe = redact_secrets(message);
    writeln!(file, "{} {}", chrono::Utc::now().to_rfc3339(), safe)?;
    Ok(())
}

pub fn log_router_decision(task: &str, provider: &str, model: &str, reason: &str) {
    tracing::info!(
        target: "neph_router",
        task = task,
        provider = provider,
        model = model,
        reason = %redact_secrets(reason),
        "model router decision"
    );
}

pub fn log_network_egress(url: &str, allowed: bool, reason: &str) {
    tracing::info!(
        target: "neph_net",
        url = %redact_secrets(url),
        allowed = allowed,
        reason = %redact_secrets(reason),
        "network egress decision"
    );
}
