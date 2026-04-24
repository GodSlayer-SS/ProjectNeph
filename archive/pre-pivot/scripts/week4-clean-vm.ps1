<#
.SYNOPSIS
  Week-4 cold-install preflight: WebView2 Evergreen registry probe (same keys as src-tauri/src/webview2.rs).

.NOTES
  Run on the target machine (or VM) after installing the Neph build. Does not install WebView2.
#>
$ErrorActionPreference = "Stop"

$min = "118.0.0.0"
$keys = @(
    "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7CFF33}",
    "HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7CFF33}"
)

$pv = $null
foreach ($k in $keys) {
    if (Test-Path -LiteralPath $k) {
        $pv = (Get-ItemProperty -LiteralPath $k).pv
        if ($pv) { break }
    }
}

if (-not $pv) {
    Write-Host "FAIL: WebView2 Evergreen registry pv not found. Install WebView2 Runtime from https://go.microsoft.com/fwlink/?linkid=2124701" -ForegroundColor Red
    exit 1
}

Write-Host "OK: WebView2 pv=$pv (minimum documented in app: $min)" -ForegroundColor Green
Write-Host "Next: install Neph from the NSIS build and run the manual checklist in docs/WEEK4_CLEAN_VM.md" -ForegroundColor Cyan
exit 0
