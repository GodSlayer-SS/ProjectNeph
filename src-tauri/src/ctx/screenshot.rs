#![allow(dead_code)]

use anyhow::Result;
use std::process::Command;

pub fn capture_screen_png() -> Result<Vec<u8>> {
    let file = tempfile::Builder::new()
        .prefix("neph-screen-")
        .suffix(".png")
        .tempfile()?;
    let path = file.path().to_string_lossy().to_string();
    let script = format!(
        r#"
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
$bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$bitmap = New-Object System.Drawing.Bitmap $bounds.Width, $bounds.Height
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)
$graphics.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
$bitmap.Save("{path}", [System.Drawing.Imaging.ImageFormat]::Png)
$graphics.Dispose()
$bitmap.Dispose()
"#
    );
    let status = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(script)
        .status()?;
    if !status.success() {
        anyhow::bail!("powershell screenshot capture failed");
    }
    let bytes = std::fs::read(file.path())?;
    Ok(bytes)
}

pub fn capture_region_png(_x: i32, _y: i32, _w: u32, _h: u32) -> Result<Vec<u8>> {
    capture_screen_png()
}
