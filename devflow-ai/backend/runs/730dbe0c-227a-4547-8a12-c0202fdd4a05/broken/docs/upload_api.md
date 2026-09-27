# UploadLab API Documentation

## Endpoint

```
POST /api/upload
Content-Type: multipart/form-data
```

## Required Field

| Field | Type | Description |
|-------|------|-------------|
| `files` | File[] | One or more files to upload. **The field name must be `files` (plural).** |

## Supported Formats

- `.txt` — plain text files
- `.png` — PNG images

## Size Limits

| Limit | Value |
|-------|-------|
| Individual file | 5 MB |
| Combined request | 10 MB |

## Success Response (200 OK)

```json
{
  "uploaded": [
    {
      "filename": "document.txt",
      "size_bytes": 1024,
      "status": "accepted"
    },
    {
      "filename": "diagram.png",
      "size_bytes": 2048,
      "status": "accepted"
    }
  ],
  "count": 2
}
```

### Response Schema

| Field | Type | Description |
|-------|------|-------------|
| `uploaded` | Array | Result entry for **every** accepted file. No file is silently omitted. |
| `count` | Integer | Number of accepted files. Must equal `uploaded.length`. |

### Uploaded File Object

| Field | Type | Description |
|-------|------|-------------|
| `filename` | String | Original filename |
| `size_bytes` | Integer | File size in bytes |
| `status` | String | Always `"accepted"` for valid files |

## Error Responses

### 400 Bad Request

Returned when no files are provided.

```json
{ "error": "No files uploaded" }
```

### 422 Unprocessable Entity

Returned when the batch contains an unsupported file format or a file exceeds the size limit.

```json
{ "error": "Unsupported file format: .exe" }
```

## Partial Validation Failures

If any file in the batch is invalid (unsupported extension or exceeds size limit), the **entire request is rejected**.

No partial uploads are accepted.

## Notes

- The multipart field name **must** be `files` — using `file` (singular) is a contract violation.
- All uploaded files must appear in the response. Silent omission is a defect.
- The `count` field must always equal `uploaded.length`.
