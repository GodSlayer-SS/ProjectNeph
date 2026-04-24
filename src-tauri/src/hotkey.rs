//! Parse persisted palette hotkey specs into `global_hotkey::HotKey`.
//!
//! Supported forms (case-insensitive, spaces optional): `ctrl+space`, `ctrl+shift+space`, `alt+space`.

use global_hotkey::hotkey::{Code, HotKey, Modifiers};

fn code_from_token(token: &str) -> Result<Code, String> {
    match token.trim().to_lowercase().as_str() {
        "space" | "spacebar" => Ok(Code::Space),
        other => Err(format!("unsupported key token: {other}")),
    }
}

pub fn parse_hotkey(spec: &str) -> Result<HotKey, String> {
    let normalized: String = spec.chars().filter(|c| !c.is_whitespace()).collect();
    let parts: Vec<&str> = normalized.split('+').collect();
    if parts.len() < 2 {
        return Err("hotkey must include a modifier and a key, e.g. ctrl+space".into());
    }
    let key_token = *parts.last().ok_or_else(|| "missing key".to_string())?;
    let code = code_from_token(key_token)?;
    let mut mods = Modifiers::default();
    for p in &parts[..parts.len() - 1] {
        match p.to_lowercase().as_str() {
            "ctrl" | "control" => mods |= Modifiers::CONTROL,
            "shift" => mods |= Modifiers::SHIFT,
            "alt" => mods |= Modifiers::ALT,
            other => return Err(format!("unsupported modifier: {other}")),
        }
    }
    if mods.is_empty() {
        return Err("at least one modifier (ctrl, shift, alt) is required".into());
    }
    Ok(HotKey::new(Some(mods), code))
}

/// Presets shown in Settings (values stored in `settings.palette_hotkey`).
pub const PRESET_SPECS: &[&str] = &["ctrl+space", "ctrl+shift+space", "alt+space"];

#[cfg(test)]
mod tests {
    use super::*;
    use global_hotkey::hotkey::Modifiers;

    #[test]
    fn parses_ctrl_space() {
        let hk = parse_hotkey("ctrl+ space").unwrap();
        assert_eq!(hk.mods, Modifiers::CONTROL);
        assert_eq!(hk.key, Code::Space);
    }

    #[test]
    fn parses_ctrl_shift_space() {
        let hk = parse_hotkey("Ctrl+Shift+Space").unwrap();
        assert!(hk.mods.contains(Modifiers::CONTROL | Modifiers::SHIFT));
        assert_eq!(hk.key, Code::Space);
    }
}
