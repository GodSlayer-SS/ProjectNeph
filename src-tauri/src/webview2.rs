//! Best-effort WebView2 Evergreen runtime version from the Edge Update client registry key.

pub const MIN_RECOMMENDED_VERSION: &str = "118.0.0.0";
pub const INSTALL_URL: &str = "https://go.microsoft.com/fwlink/?linkid=2124701";

#[cfg(windows)]
const WEBVIEW_CLIENT_KEY: &str =
    r"SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7CFF33}";

#[cfg(windows)]
const WEBVIEW_CLIENT_KEY_32: &str =
    r"SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7CFF33}";

#[cfg(windows)]
pub fn evergreen_runtime_version() -> Option<String> {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    for subkey in [WEBVIEW_CLIENT_KEY, WEBVIEW_CLIENT_KEY_32] {
        if let Ok(k) = hklm.open_subkey(subkey) {
            if let Ok(pv) = k.get_value::<String, _>("pv") {
                if !pv.is_empty() {
                    return Some(pv);
                }
            }
        }
    }
    None
}

#[cfg(not(windows))]
pub fn evergreen_runtime_version() -> Option<String> {
    None
}

fn version_components(s: &str) -> Vec<u32> {
    s.split(['.', ','])
        .filter_map(|p| p.parse::<u32>().ok())
        .collect()
}

/// Lexicographic compare on dotted numeric components (shorter vectors padded with 0).
pub fn version_ge(installed: &str, minimum: &str) -> bool {
    let mut a = version_components(installed);
    let mut b = version_components(minimum);
    let n = a.len().max(b.len());
    a.resize(n, 0);
    b.resize(n, 0);
    a >= b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions() {
        assert!(version_ge("120.0.2535.51", MIN_RECOMMENDED_VERSION));
        assert!(!version_ge("100.0.0.0", MIN_RECOMMENDED_VERSION));
    }
}
