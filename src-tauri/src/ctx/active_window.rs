use std::process::Command;

fn query_active_window() -> Option<(String, String)> {
    let script = r#"
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Win32 {
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern int GetWindowText(IntPtr hWnd, System.Text.StringBuilder text, int count);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint processId);
}
"@
$h = [Win32]::GetForegroundWindow()
$sb = New-Object System.Text.StringBuilder 512
[void][Win32]::GetWindowText($h, $sb, $sb.Capacity)
$pid = 0
[void][Win32]::GetWindowThreadProcessId($h, [ref]$pid)
$name = ""
try { $name = (Get-Process -Id $pid -ErrorAction Stop).ProcessName } catch {}
Write-Output ("{0}|{1}" -f $sb.ToString(), $name)
"#;
    let output = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(script)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let mut parts = text.splitn(2, '|');
    let title = parts.next().unwrap_or_default().trim().to_string();
    let proc = parts.next().unwrap_or_default().trim().to_string();
    Some((title, proc))
}

pub fn active_window_title() -> String {
    query_active_window()
        .map(|(title, _)| if title.is_empty() { "unknown".into() } else { title })
        .unwrap_or_else(|| "unknown".into())
}

pub fn active_process_name() -> String {
    query_active_window()
        .map(|(_, proc)| if proc.is_empty() { "unknown".into() } else { proc })
        .unwrap_or_else(|| "unknown".into())
}
