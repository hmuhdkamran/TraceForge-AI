# UploadLab — Authoritative Acceptance Criteria & API Contract Specification

**Module:** UploadLab  
**Version:** 1.0.0  
**Contract ID:** CONTRACT-UPLOADLAB-001  
**Authoritative Specification:** `sample_project/broken/openapi.yaml`  

---

## 1. System Requirements Overview

UploadLab is a multi-file upload service designed for developer workflows. It exposes a single HTTP POST endpoint for accepting, validating, and recording uploaded files.

```
POST /api/upload
Content-Type: multipart/form-data
```

---

## 2. Acceptance Criteria

### AC-001: Multi-File Batch Ingestion (REQ-001)
- **Given** an HTTP client prepares a multipart/form-data request containing multiple files (2 or more)
- **When** the request is submitted to `POST /api/upload`
- **Then** the backend must iterate over and process **every** file entry in the multipart stream
- **And** the returned `count` field must equal the exact number of files submitted
- **And** the `uploaded` array must contain an entry for each submitted file with `status: "accepted"`
- **And** no file may be silently dropped, ignored, or omitted from processing or the response payload.

### AC-002: Multipart Field Name Conformance (REQ-002)
- **Given** the frontend or any client client submits multipart form data
- **When** files are appended to the payload
- **Then** each file part MUST use the field name `files` (plural) as specified in OpenAPI
- **And** requests using singular `file` or other non-conforming field names are considered contract violations.

### AC-003: Single File Ingestion (REQ-003)
- **Given** a user selects exactly one valid file (.txt or .png <= 5 MB)
- **When** the form is submitted
- **Then** the response status must be `200 OK`
- **And** the JSON response must have `count: 1` and `uploaded.length == 1`.

### AC-004: File Extension & MIME Whitelist Enforcement (REQ-004)
- **Given** an upload batch contains a file with an unsupported file extension (not `.txt` or `.png`)
- **When** the request is processed by the server
- **Then** the server must reject the batch with HTTP `422 Unprocessable Entity`
- **And** return a structured JSON error body: `{"error": "Unsupported file format: .<ext>"}`
- **And** no partial state or file persistence shall occur.

### AC-005: Individual File Size Limit Enforcement (REQ-005)
- **Given** any individual file in the upload batch exceeds 5 MB (5,242,880 bytes)
- **When** the request is processed
- **Then** the server must reject the request with HTTP `422 Unprocessable Entity`
- **And** return a structured error message indicating the file exceeds the 5 MB limit.

### AC-006: Combined Request Size Limit Enforcement (REQ-006)
- **Given** the combined payload size of all uploaded files exceeds 10 MB (10,485,760 bytes)
- **When** the request is received by the server
- **Then** the server must reject the request with HTTP `422 Unprocessable Entity` or `413 Payload Too Large`.

### AC-007: Empty Request Rejection (REQ-007)
- **Given** a request reaches `POST /api/upload` with an empty multipart body (zero files attached)
- **When** the handler evaluates the stream
- **Then** the server must return HTTP `400 Bad Request`
- **And** return a structured error message: `{"error": "No files uploaded"}`.

### AC-008: Canonical Response Schema (REQ-008)
- **Given** a successful upload request
- **When** the HTTP response is generated
- **Then** it must strictly adhere to the OpenAPI `UploadResponse` schema:
  ```json
  {
    "uploaded": [
      {
        "filename": "string",
        "size_bytes": 1024,
        "status": "accepted"
      }
    ],
    "count": 1
  }
  ```
- **And** the Content-Type header must be `application/json`.

### AC-009: Atomic Batch Validation (REQ-009)
- **Given** an upload batch containing both valid files and invalid files (e.g., one `.txt` and one `.exe`)
- **When** processed by the server
- **Then** the entire batch must be rejected atomically with `422 Unprocessable Entity`
- **And** the server must not return `200 OK` with partial results.

---

## 3. Deliberate Inconsistencies in Broken Fixture

The testbed fixture in `sample_project/broken` introduces two deliberate defects to test ContractGuard's diagnostic capabilities:

1. **BUG-001 (Frontend Request Defect):**  
   `sample_project/broken/frontend/src/api.ts` constructs FormData using `'file'` instead of `'files'`, violating AC-002.
2. **BUG-002 (Backend Processing Defect):**  
   `sample_project/broken/backend/src/main.rs` processes only `if let Some(field) = multipart.next_field()` instead of `while let Some(field) = multipart.next_field()`, violating AC-001, AC-008, AC-009.
