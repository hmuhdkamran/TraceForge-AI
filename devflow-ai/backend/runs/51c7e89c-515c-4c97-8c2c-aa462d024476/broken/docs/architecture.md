# UploadLab — Architecture

## Overview

UploadLab is a minimal full-stack file upload application used as a test fixture for TraceForge AI ContractGuard.

## Components

```
uploadlab/
├── frontend/   React + TypeScript (Vite)
│   └── src/
│       ├── api.ts              API client (multipart/form-data construction)
│       └── components/
│           └── UploadForm.tsx  Upload UI component
└── backend/    Rust + Axum
    └── src/
        └── main.rs             HTTP server with /api/upload endpoint
```

## Data Flow

```
User selects files
      │
      ▼
UploadForm.tsx
      │  calls uploadFiles()
      ▼
api.ts
      │  POST /api/upload
      │  multipart/form-data
      │  field name: "files"      ← documented contract
      ▼
Axum handler (main.rs)
      │  reads multipart stream
      │  validates each file
      │  returns JSON response
      ▼
UploadResponse { uploaded[], count }
```

## API Contract

The authoritative contract is defined in `openapi.yaml`.

The multipart field name **must** be `files`.

The response must include every uploaded file — no silent omissions.

## Known Defects (Broken Fixture)

This fixture intentionally contains two bugs for use in ContractGuard investigations:

| ID | Location | Description |
|----|----------|-------------|
| BUG-001 | `frontend/src/api.ts` | Uses field name `file` instead of `files` |
| BUG-002 | `backend/src/main.rs` | Processes only the first file; ignores subsequent files |

These bugs are deliberate and must not be corrected in the broken fixture.
