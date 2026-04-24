pub fn clipboard_preview() -> String {
    let mut clipboard = match arboard::Clipboard::new() {
        Ok(c) => c,
        Err(_) => return "clipboard unavailable".into(),
    };
    match clipboard.get_text() {
        Ok(text) => text.chars().take(240).collect(),
        Err(_) => "clipboard empty".into(),
    }
}
