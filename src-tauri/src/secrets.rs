use anyhow::Result;

const SERVICE_PREFIX: &str = "Neph";
const ALLOWED_PROVIDERS: &[&str] = &["groq", "gemini", "openrouter", "anthropic"];

fn assert_allowed_provider(provider: &str) -> Result<()> {
    if ALLOWED_PROVIDERS.contains(&provider) {
        return Ok(());
    }
    anyhow::bail!("provider is not allowed")
}

pub fn save_provider_key(provider: &str, key: &str) -> Result<()> {
    assert_allowed_provider(provider)?;
    if key.trim().is_empty() {
        anyhow::bail!("refusing to store empty API key");
    }
    let entry = keyring::Entry::new(&format!("{SERVICE_PREFIX}.{provider}"), "default")?;
    entry.set_password(key).map_err(|e| {
        anyhow::anyhow!(
            "could not store API key in OS credential store (refusing plaintext fallback): {e}"
        )
    })?;
    Ok(())
}

pub fn read_provider_key(provider: &str) -> Result<Option<String>> {
    assert_allowed_provider(provider)?;
    let entry = keyring::Entry::new(&format!("{SERVICE_PREFIX}.{provider}"), "default")?;
    match entry.get_password() {
        Ok(value) => Ok(Some(value)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_key() {
        assert!(save_provider_key("groq", "").is_err());
    }

    #[test]
    fn rejects_disallowed_provider() {
        assert!(save_provider_key("evil", "x").is_err());
    }
}
