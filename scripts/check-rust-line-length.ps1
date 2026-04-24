# Fail if any Rust source under src-tauri/src (except target) exceeds $MaxLines.
# Exclusions: none by default; extend as needed for generated code.
param(
    [int]$MaxLines = 500
)

$ErrorActionPreference = "Stop"
$root = Join-Path (Join-Path (Join-Path $PSScriptRoot "..") "src-tauri") "src"
if (-not (Test-Path $root)) {
    Write-Error "Expected Rust src at $root"
}

$failed = @()
Get-ChildItem -Path $root -Filter "*.rs" -Recurse | ForEach-Object {
    $lines = (Get-Content $_.FullName | Measure-Object -Line).Lines
    if ($lines -gt $MaxLines) {
        $failed += "$($_.FullName): $lines lines (max $MaxLines)"
    }
}

if ($failed.Count -gt 0) {
    Write-Host "Rust file length guard failed:" -ForegroundColor Red
    $failed | ForEach-Object { Write-Host "  $_" }
    exit 1
}

Write-Host "Rust file length OK (max $MaxLines lines per file under $root)."
