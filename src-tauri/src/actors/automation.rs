/// actors/automation.rs — Desktop automation (Blueprint Phase 3).
///
/// Tools: `focus_window`, `type_in_active`, `read_active`.
///
/// The Blueprint names `uiautomation` + `enigo`. On Windows the runner uses
/// **WScript.Shell** (`AppActivate`, `SendKeys`) via PowerShell — reliable on
/// typical desktops without extra native deps. `read_active` uses `ctx::collect_snapshot`.
/// UIAutomation/enigo remain a possible upgrade behind the same tool API.

/// Active window title, process name, and clipboard preview (palette `read_active`).
pub fn desktop_read_active() -> String {
    let snapshot = crate::ctx::collect_snapshot();
    format!(
        "Active window: {}\nActive process: {}\nClipboard: {}",
        snapshot.active_window_title,
        snapshot.active_process_name,
        snapshot.clipboard_preview
    )
}

/// Bring a window to the foreground by title substring (Windows).
pub fn desktop_focus_window(query: &str) -> String {
    let query = query.trim();
    if query.is_empty() {
        return "Usage: >focus <window title/process>".to_string();
    }
    #[cfg(windows)]
    {
        let script = format!(
            "$ws=New-Object -ComObject WScript.Shell; if($ws.AppActivate('{q}')){{'focused'}} else {{'not_found'}}",
            q = query.replace('\'', "''")
        );
        let Ok(output) = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output()
        else {
            return format!("PowerShell failed while focusing '{query}'");
        };
        let out = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if out.contains("focused") {
            format!("Focused window matching '{query}'")
        } else {
            format!("No matching window found for '{query}'")
        }
    }
    #[cfg(not(windows))]
    {
        let _ = query;
        "focus_window is currently Windows-only".to_string()
    }
}

/// Send keystrokes to the foreground window (Windows). Yellow-tier: gated by trust kernel.
pub fn desktop_type_in_active(text: &str) -> String {
    if text.trim().is_empty() {
        return "Usage: >type <text>".to_string();
    }
    #[cfg(windows)]
    {
        let escaped = text
            .replace("{", "{{}")
            .replace("}", "{}}")
            .replace("+", "{+}")
            .replace("^", "{^}")
            .replace("%", "{%}")
            .replace("~", "{~}");
        let script = format!(
            "$ws=New-Object -ComObject WScript.Shell; Start-Sleep -Milliseconds 80; $ws.SendKeys('{t}'); 'typed'",
            t = escaped.replace('\'', "''")
        );
        let _ = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output();
        "Sent keystrokes to active window.".to_string()
    }
    #[cfg(not(windows))]
    {
        let _ = text;
        "type_in_active is currently Windows-only".to_string()
    }
}

pub struct AutomationActor;

impl AutomationActor {
    pub fn new() -> Self {
        Self
    }

    pub fn focus_window(&self, query: &str) -> anyhow::Result<String> {
        Ok(desktop_focus_window(query))
    }

    pub fn type_in_active(&self, text: &str) -> anyhow::Result<()> {
        let msg = desktop_type_in_active(text);
        if msg.starts_with("Usage:") || msg.contains("Windows-only") {
            anyhow::bail!("{msg}");
        }
        Ok(())
    }

    pub fn read_active(&self) -> anyhow::Result<String> {
        Ok(desktop_read_active())
    }
}

impl Default for AutomationActor {
    fn default() -> Self {
        Self::new()
    }
}
