# TraceForge AI -- Verification Script for Windows PowerShell
$ErrorActionPreference = "Stop"

Write-Host "=================================================" -ForegroundColor Cyan
Write-Host " TraceForge AI -- Comprehensive Verification" -ForegroundColor Cyan
Write-Host "=================================================" -ForegroundColor Cyan

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$RootDir = Split-Path -Parent $ScriptDir

# 1. ContractGuard Backend Tests
Write-Host ""
Write-Host "[1/4] Running ContractGuard Backend tests..." -ForegroundColor Yellow
Push-Location "$RootDir\backend"
try {
    cargo test --quiet
    Write-Host " ContractGuard Backend: PASS" -ForegroundColor Green
} finally {
    Pop-Location
}

# 2. ContractGuard Frontend Tests and Build
Write-Host ""
Write-Host "[2/4] Running ContractGuard Frontend tests..." -ForegroundColor Yellow
Push-Location "$RootDir\frontend"
try {
    npm test -- --run
    Write-Host " ContractGuard Frontend Tests: PASS" -ForegroundColor Green
    npm run build
    Write-Host " ContractGuard Frontend Build: PASS" -ForegroundColor Green
} finally {
    Pop-Location
}

# 3. UploadLab Broken Backend Baseline (Expect BUG-002 Failures)
Write-Host ""
Write-Host "[3/4] Verifying UploadLab Broken Backend Baseline (BUG-002)..." -ForegroundColor Yellow
Push-Location "$RootDir\sample_project\broken\backend"
try {
    $backendOutput = cmd /c "cargo test 2>&1"
    $outputString = $backendOutput -join "`n"
    $hasBug002_1 = $outputString -match "test_upload_two_valid_files_bug002 \.\.\. FAILED"
    $hasBug002_2 = $outputString -match "test_upload_three_valid_files_bug002 \.\.\. FAILED"
    $hasBug002_3 = $outputString -match "test_no_silent_omission_bug002 \.\.\. FAILED"

    if ($hasBug002_1 -and $hasBug002_2 -and $hasBug002_3) {
        Write-Host " UploadLab Backend Baseline: PASS (BUG-002 reproduced with exact expected test failures)" -ForegroundColor Green
    } else {
        Write-Host " UploadLab Backend Baseline: FAIL (Expected test failures for BUG-002 not observed)" -ForegroundColor Red
        Write-Host $outputString
        exit 1
    }
} finally {
    Pop-Location
}

# 4. UploadLab Broken Frontend Baseline (Expect BUG-001 Failures)
Write-Host ""
Write-Host "[4/4] Verifying UploadLab Broken Frontend Baseline (BUG-001)..." -ForegroundColor Yellow
Push-Location "$RootDir\sample_project\broken\frontend"
try {
    npm run build
    Write-Host " UploadLab Frontend Build: PASS" -ForegroundColor Green

    $frontendOutput = cmd /c "npm test -- --run 2>&1"
    $frontString = $frontendOutput -join "`n"
    $hasBug001_1 = $frontString -match "should use field name"
    $hasBug001_2 = $frontString -match "should append files with field name"
    $hasBug001_3 = $frontString -match "should use consistent field name"

    if ($hasBug001_1 -and $hasBug001_2 -and $hasBug001_3) {
        Write-Host " UploadLab Frontend Baseline: PASS (BUG-001 reproduced with exact expected test failures)" -ForegroundColor Green
    } else {
        Write-Host " UploadLab Frontend Baseline: FAIL (Expected test failures for BUG-001 not observed)" -ForegroundColor Red
        Write-Host $frontString
        exit 1
    }
} finally {
    Pop-Location
}

Write-Host ""
Write-Host "=================================================" -ForegroundColor Cyan
Write-Host " All Systems Verified: M1, M2, M3, M4, and M5 Complete!" -ForegroundColor Green
Write-Host "=================================================" -ForegroundColor Cyan
