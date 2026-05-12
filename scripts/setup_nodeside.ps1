# Nephis Nodeside Sidecar Setup
# Blueprint Phase 3: Playwright browser automation with 4-profile isolation
#
# Run this script ONCE before first use of browser tools.
# Requires: Node.js 20+ installed and on PATH.
#
# Usage:
#   .\scripts\setup_nodeside.ps1

$ErrorActionPreference = "Stop"

Write-Host "=== Nephis Nodeside Sidecar Setup ===" -ForegroundColor Cyan

# 1. Install npm dependencies
$nodesideDir = Join-Path $PSScriptRoot "..\apps\nodeside"
Write-Host "`n[1/3] Installing npm packages in $nodesideDir ..."
Push-Location $nodesideDir
try {
    npm install
} finally {
    Pop-Location
}

# 2. Install Playwright Chromium browser
Write-Host "`n[2/3] Installing Playwright Chromium (this downloads ~150 MB) ..."
Push-Location $nodesideDir
try {
    npx playwright install chromium
} finally {
    Pop-Location
}

# 3. Create browser profile directories
$profileBase = Join-Path $env:USERPROFILE ".nephis\browser-profiles"
$profiles = @(
    "nephis-research",
    "nephis-tools",
    "nephis-personal",
    "nephis-throwaway"
)
Write-Host "`n[3/3] Creating browser profile directories in $profileBase ..."
foreach ($profile in $profiles) {
    $dir = Join-Path $profileBase $profile
    if (-not (Test-Path $dir)) {
        New-Item -ItemType Directory -Path $dir | Out-Null
        Write-Host "  Created: $dir"
    } else {
        Write-Host "  Exists:  $dir"
    }
}

Write-Host "`n=== Setup Complete ===" -ForegroundColor Green
Write-Host "Browser tools (browser_read_page, browser_search, etc.) are now available."
Write-Host "Start the nodeside server with: node apps\nodeside\server.js"
Write-Host ""
Write-Host "Profile isolation summary:"
Write-Host "  nephis-research  — anonymous browsing, research (Green/Yellow)"
Write-Host "  nephis-tools     — form filling, automation (Yellow)"
Write-Host "  nephis-personal  — logged-in sessions (Red — explicit only)"
Write-Host "  nephis-throwaway — sandboxed, discarded after session (Green)"
