use anyhow::Result;
use std::io::Write;
use std::process::Command;

pub fn extract_text_from_png_bytes(_png_bytes: &[u8]) -> Result<String> {
    let mut image = tempfile::Builder::new()
        .prefix("neph-ocr-")
        .suffix(".png")
        .tempfile()?;
    image.write_all(_png_bytes)?;
    let out_base = tempfile::Builder::new()
        .prefix("neph-ocr-out-")
        .tempfile()?;
    let out_path = out_base.path().to_string_lossy().to_string();
    let status = Command::new("tesseract")
        .arg(image.path())
        .arg(&out_path)
        .arg("-l")
        .arg("eng")
        .status();
    match status {
        Ok(s) if s.success() => {
            let txt_path = format!("{out_path}.txt");
            let text = std::fs::read_to_string(txt_path)?;
            Ok(text)
        }
        _ => Ok(String::new()),
    }
}
