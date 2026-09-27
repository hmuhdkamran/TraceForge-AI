#!/usr/bin/env bash
set -e

echo "================================================="
echo " TraceForge AI — Comprehensive Verification"
echo "================================================="

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

# 1. ContractGuard Backend Tests
echo ""
echo "[1/4] Running ContractGuard Backend tests..."
cd "${ROOT_DIR}/backend"
cargo test --quiet
echo " ContractGuard Backend: PASS"

# 2. ContractGuard Frontend Tests & Build
echo ""
echo "[2/4] Running ContractGuard Frontend tests..."
cd "${ROOT_DIR}/frontend"
npm test -- --run
echo " ContractGuard Frontend Tests: PASS"
npm run build
echo " ContractGuard Frontend Build: PASS"

# 3. UploadLab Broken Backend Baseline (Expect BUG-002 Failures)
echo ""
echo "[3/4] Verifying UploadLab Broken Backend Baseline (BUG-002)..."
cd "${ROOT_DIR}/sample_project/broken/backend"
set +e
BACKEND_OUTPUT=$(cargo test 2>&1)
set -e

if echo "${BACKEND_OUTPUT}" | grep -q "test_upload_two_valid_files_bug002 \.\.\. FAILED" && \
   echo "${BACKEND_OUTPUT}" | grep -q "test_upload_three_valid_files_bug002 \.\.\. FAILED" && \
   echo "${BACKEND_OUTPUT}" | grep -q "test_no_silent_omission_bug002 \.\.\. FAILED"; then
    echo " UploadLab Backend Baseline: PASS (BUG-002 reproduced with exact expected test failures)"
else
    echo " UploadLab Backend Baseline: FAIL (Expected test failures for BUG-002 not observed)"
    exit 1
fi

# 4. UploadLab Broken Frontend Baseline (Expect BUG-001 Failures)
echo ""
echo "[4/4] Verifying UploadLab Broken Frontend Baseline (BUG-001)..."
cd "${ROOT_DIR}/sample_project/broken/frontend"
npm run build
echo " UploadLab Frontend Build: PASS"

set +e
FRONTEND_OUTPUT=$(npm test -- --run 2>&1)
set -e

if echo "${FRONTEND_OUTPUT}" | grep -q "should use field name" && \
   echo "${FRONTEND_OUTPUT}" | grep -q "should append files with field name" && \
   echo "${FRONTEND_OUTPUT}" | grep -q "should use consistent field name"; then
    echo " UploadLab Frontend Baseline: PASS (BUG-001 reproduced with exact expected test failures)"
else
    echo " UploadLab Frontend Baseline: FAIL (Expected test failures for BUG-001 not observed)"
    exit 1
fi

echo ""
echo "================================================="
echo " All Systems Verified: M1 & M2 Complete!"
echo "================================================="
