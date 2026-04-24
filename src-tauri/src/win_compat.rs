//! Windows-specific file metadata and clearer I/O error hints.

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

#[cfg(windows)]
const FILE_ATTRIBUTE_OFFLINE: u32 = 0x1000;
#[cfg(windows)]
const FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS: u32 = 0x400_000;

/// True when the file looks like a OneDrive (or similar) placeholder that is not fully local.
#[cfg(windows)]
pub fn is_cloud_placeholder_metadata(meta: &std::fs::Metadata) -> bool {
    let attr = meta.file_attributes();
    (attr & FILE_ATTRIBUTE_OFFLINE) != 0 || (attr & FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS) != 0
}

#[cfg(not(windows))]
pub fn is_cloud_placeholder_metadata(_meta: &std::fs::Metadata) -> bool {
    false
}

const CONTROLLED_FOLDER_HINT: &str = " If Windows Defender Controlled folder access is enabled, allow Neph under \
     Windows Security → Virus & threat protection → Ransomware protection → \
     Allow an app through Controlled folder access.";

pub fn format_io_error(err: std::io::Error, operation: &str) -> String {
    let mut msg = format!("{operation}: {err}");
    if err.raw_os_error() == Some(5) {
        msg.push_str(CONTROLLED_FOLDER_HINT);
    }
    msg
}

/// Best-effort hint when a non-`io::Error` still looks like access denied (e.g. `trash` errors).
pub fn format_access_like_error(message: impl AsRef<str>) -> String {
    let text = message.as_ref();
    let lower = text.to_lowercase();
    let mut out = text.to_string();
    if lower.contains("access denied")
        || lower.contains("permission denied")
        || lower.contains("refused")
        || lower.contains("not allowed")
    {
        out.push_str(CONTROLLED_FOLDER_HINT);
    }
    out
}
