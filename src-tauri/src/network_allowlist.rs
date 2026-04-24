const ALLOWED_HOSTS: &[&str] = &[
    "duckduckgo.com",
    "html.duckduckgo.com",
    "github.com",
    "docs.rs",
    "stackoverflow.com",
    "example.com",
];

pub fn host_allowed(url: &str) -> bool {
    let parsed = match url::Url::parse(url) {
        Ok(v) => v,
        Err(_) => return false,
    };
    if !matches!(parsed.scheme(), "http" | "https") {
        return false;
    }
    let Some(host) = parsed.host_str() else {
        return false;
    };
    ALLOWED_HOSTS.iter().any(|allowed| host == *allowed || host.ends_with(&format!(".{allowed}")))
}
