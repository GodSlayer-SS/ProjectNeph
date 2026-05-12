<#
.SYNOPSIS
  Run Neph quality checks with per-step timeouts and live console output.

.NOTES
  - Uses a fresh CARGO_TARGET_DIR each run to avoid fighting other cargo/rustc locks.
  - Do NOT pipe live cargo output through `Select-Object -Last N` in PowerShell: it buffers
    the entire stream until cargo exits, so the terminal looks "stuck with no output".
  - This script uses Start-Process so cargo writes directly to the same console.
#>
param(
    [int]$TypecheckTimeoutSec = 180,
    [int]$ClippyTimeoutSec = 600,
    [int]$TestTimeoutSec = 900
)

$ErrorActionPreference = "Stop"
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$cargoRoot = Join-Path $repoRoot "src-tauri"
$targetDir = Join-Path ([System.IO.Path]::GetTempPath()) ("neph-cargo-" + [Guid]::NewGuid().ToString("n"))
New-Item -ItemType Directory -Force -Path $targetDir | Out-Null
$env:CARGO_TARGET_DIR = $targetDir
$cargoExe = (Get-Command cargo -ErrorAction Stop).Source
$cmdExe = (Get-Command cmd.exe -ErrorAction Stop).Source

Write-Host "=== Neph verify ===" -ForegroundColor Cyan
Write-Host "CARGO_TARGET_DIR=$targetDir" -ForegroundColor Cyan
Write-Host ""

function Invoke-ProcessWithTimeout {
    param(
        [string]$StepName,
        [string]$FilePath,
        [string[]]$Arguments,
        [string]$WorkingDirectory,
        [int]$TimeoutSec
    )
    Write-Host "[$StepName] $FilePath $($Arguments -join ' ')" -ForegroundColor Green
    $p = Start-Process -FilePath $FilePath -ArgumentList $Arguments -WorkingDirectory $WorkingDirectory `
        -NoNewWindow -PassThru
    $ms = [int]([Math]::Min([int]::MaxValue, [long]$TimeoutSec * 1000L))
    if (-not $p.WaitForExit($ms)) {
        Write-Host ""
        Write-Host "[$StepName] TIMEOUT after ${TimeoutSec}s." -ForegroundColor Red
        Write-Host "Stop other cargo/IDE builds, or run with a fresh CARGO_TARGET_DIR (this script already uses one)." -ForegroundColor Yellow
        try { $p.Kill() } catch { }
        exit 124
    }
    $p.Refresh()
    $exitCode = 0
    if ($null -ne $p.ExitCode) {
        $exitCode = [int]$p.ExitCode
    }
    if ($exitCode -ne 0) {
        Write-Host "[$StepName] failed with exit code $exitCode" -ForegroundColor Red
        exit $exitCode
    }
    Write-Host "[$StepName] OK" -ForegroundColor Green
}

Write-Host "[1/4] Rust file length guard..." -ForegroundColor Green
& (Join-Path $PSScriptRoot "check-rust-line-length.ps1")
if (-not $?) {
    exit $(if ($null -ne $LASTEXITCODE) { $LASTEXITCODE } else { 1 })
}

Write-Host "[2/4] npm run typecheck..." -ForegroundColor Green
Invoke-ProcessWithTimeout -StepName "typecheck" -FilePath $cmdExe -Arguments @("/c", "npm run typecheck") `
    -WorkingDirectory $repoRoot -TimeoutSec $TypecheckTimeoutSec

Write-Host "[3/4] cargo clippy --all-targets (-D warnings)..." -ForegroundColor Green
Invoke-ProcessWithTimeout -StepName "clippy" -FilePath $cargoExe -Arguments @(
    "clippy", "--all-targets", "--", "-Dwarnings"
) -WorkingDirectory $cargoRoot -TimeoutSec $ClippyTimeoutSec

Write-Host "[4/4] cargo test..." -ForegroundColor Green
Invoke-ProcessWithTimeout -StepName "cargo test" -FilePath $cargoExe -Arguments @("test") `
    -WorkingDirectory $cargoRoot -TimeoutSec $TestTimeoutSec

Write-Host ""
Write-Host "=== All verify steps completed OK ===" -ForegroundColor Cyan
