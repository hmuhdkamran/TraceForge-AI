# UploadLab — Acceptance Criteria

## AC-001: Multiple file upload

**Given** the user selects two or more files  
**When** they submit the upload form  
**Then** the response must include a result entry for every uploaded file  
**And** `count` must equal the number of files selected

## AC-002: Correct multipart field name

**Given** the frontend sends a multipart/form-data request  
**When** it attaches files  
**Then** it must use the field name `files` (plural)  
**And** the backend must accept requests using this field name

## AC-003: Single file upload

**Given** the user selects one file  
**When** they submit the upload form  
**Then** the response must contain `count: 1` and one entry in `uploaded`

## AC-004: Unsupported format rejection

**Given** the user selects a file with an unsupported extension  
**When** they submit the upload form  
**Then** the API must return 422 Unprocessable Entity  
**And** no files are partially accepted

## AC-005: File size limit

**Given** the user selects a file larger than 5 MB  
**When** they submit the upload form  
**Then** the API must return 422 Unprocessable Entity

## AC-006: Empty upload

**Given** the user submits the form with no files selected  
**When** the request reaches the API  
**Then** the API must return 400 Bad Request

## AC-007: Response schema

**Given** a successful upload  
**When** the response is received  
**Then** it must contain `uploaded` (array), `count` (integer)  
**And** each entry must have `filename`, `size_bytes`, `status`

## AC-008: No silent omission

**Given** the user uploads three files  
**When** all are valid  
**Then** all three must appear in the `uploaded` array  
**And** none may be silently omitted from the response
