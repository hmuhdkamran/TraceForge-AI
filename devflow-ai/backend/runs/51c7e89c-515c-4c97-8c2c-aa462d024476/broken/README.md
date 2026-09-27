# UploadLab

A simple multi-file upload application used as a test fixture for **TraceForge AI ContractGuard**.

## ⚠️ BROKEN FIXTURE

This directory contains deliberately broken code with two API contract defects:

| Bug | Location | Description |
|-----|----------|-------------|
| BUG-001 | `frontend/src/api.ts` | Wrong multipart field name (`file` instead of `files`) |
| BUG-002 | `backend/src/main.rs` | Backend only processes the first file |

**Do not fix these files.** They are the immutable baseline for ContractGuard investigations.

## Running

### Backend

```powershell
cd backend
cargo run
```

Listens on `http://localhost:3001`

### Frontend

```powershell
cd frontend
npm install
npm run dev
```

Listens on `http://localhost:5174`

## Running Tests

### Backend tests (expect failures for BUG-002)

```powershell
cd backend
cargo test
```

**Expected failures:**
- `test_upload_two_valid_files_bug002`
- `test_upload_three_valid_files_bug002`
- `test_no_silent_omission_bug002`

### Frontend tests (expect failures for BUG-001)

```powershell
cd frontend
npm install
npm test
```

**Expected failures:**
- `should use field name "files" as specified in the API contract`
- `should append files with field name "files" to FormData`
- `should use consistent field name for multiple files`

## Documentation

- `openapi.yaml` — authoritative API contract
- `docs/upload_api.md` — API documentation
- `docs/acceptance_criteria.md` — acceptance criteria
- `docs/architecture.md` — architecture overview
