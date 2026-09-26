// UploadLab backend — BROKEN FIXTURE
// This file intentionally contains BUG-002:
// The handler only processes the FIRST file, not all uploaded files.
// Do NOT fix this file — it is the immutable broken baseline.

use axum::{
    extract::Multipart,
    http::StatusCode,
    response::Json,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

#[derive(Debug, Serialize, Deserialize)]
struct UploadedFile {
    filename: String,
    size_bytes: usize,
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UploadResponse {
    uploaded: Vec<UploadedFile>,
    count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct ErrorResponse {
    error: String,
}

const ALLOWED_EXTENSIONS: &[&str] = &["txt", "png"];
const MAX_FILE_SIZE: usize = 5 * 1024 * 1024; // 5 MB

async fn upload_handler(mut multipart: Multipart) -> Result<Json<UploadResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut uploaded = Vec::new();

    // BUG-002: Only processes the first field, returns immediately after first file
    // Should iterate over ALL fields until multipart.next_field() returns None
    if let Some(field) = multipart.next_field().await.map_err(|e| {
        (StatusCode::UNPROCESSABLE_ENTITY, Json(ErrorResponse { error: format!("Multipart error: {}", e) }))
    })? {
        let filename = field.file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
        if !ALLOWED_EXTENSIONS.contains(&ext.as_str()) {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ErrorResponse { error: format!("Unsupported file format: .{}", ext) }),
            ));
        }

        let data = field.bytes().await.map_err(|e| {
            (StatusCode::UNPROCESSABLE_ENTITY, Json(ErrorResponse { error: format!("Read error: {}", e) }))
        })?;

        if data.len() > MAX_FILE_SIZE {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ErrorResponse { error: format!("File {} exceeds 5 MB limit", filename) }),
            ));
        }

        uploaded.push(UploadedFile {
            filename,
            size_bytes: data.len(),
            status: "accepted".to_string(),
        });
    } else {
        // No files uploaded
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "No files uploaded".to_string() }),
        ));
    }

    let count = uploaded.len();
    Ok(Json(UploadResponse { uploaded, count }))
}

pub fn router() -> Router {
    Router::new()
        .route("/api/upload", post(upload_handler))
        .layer(CorsLayer::permissive())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = router();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001").await.unwrap();
    tracing::info!("UploadLab (BROKEN) listening on 127.0.0.1:3001");
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode, header};
    use http_body_util::BodyExt;

    // Helper: build a multipart body with correct field name "files"
    fn build_multipart_correct(files: &[(&str, &[u8])]) -> (String, Vec<u8>) {
        let boundary = "----TestBoundary12345";
        let mut body = Vec::new();
        for (filename, data) in files {
            body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
            body.extend_from_slice(
                format!(
                    "Content-Disposition: form-data; name=\"files\"; filename=\"{}\"\r\nContent-Type: application/octet-stream\r\n\r\n",
                    filename
                ).as_bytes()
            );
            body.extend_from_slice(data);
            body.extend_from_slice(b"\r\n");
        }
        body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());
        (format!("multipart/form-data; boundary={}", boundary), body)
    }

    // Helper: build multipart with WRONG field name (as the broken frontend does)
    fn build_multipart_wrong_field(files: &[(&str, &[u8])]) -> (String, Vec<u8>) {
        let boundary = "----TestBoundary12345";
        let mut body = Vec::new();
        for (filename, data) in files {
            body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
            body.extend_from_slice(
                format!(
                    "Content-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\nContent-Type: application/octet-stream\r\n\r\n",
                    filename
                ).as_bytes()
            );
            body.extend_from_slice(data);
            body.extend_from_slice(b"\r\n");
        }
        body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());
        (format!("multipart/form-data; boundary={}", boundary), body)
    }

    async fn do_request(body: Vec<u8>, content_type: String) -> axum::response::Response {
        use tower::ServiceExt;
        let app = router();
        let req = Request::builder()
            .method("POST")
            .uri("/api/upload")
            .header("content-type", content_type)
            .body(Body::from(body))
            .unwrap();
        app.oneshot(req).await.unwrap()
    }

    /// TEST 01: Upload one valid file — should return 200 with count=1
    #[tokio::test]
    async fn test_upload_one_valid_file() {
        let (ct, body) = build_multipart_correct(&[("document.txt", b"hello world")]);
        let resp = do_request(body, ct).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["count"], 1);
        assert_eq!(json["uploaded"][0]["status"], "accepted");
    }

    /// TEST 02: Upload two valid files — BUG-002 causes this to fail (only 1 returned)
    /// Expected to FAIL on broken fixture
    #[tokio::test]
    async fn test_upload_two_valid_files_bug002() {
        let (ct, body) = build_multipart_correct(&[
            ("file1.txt", b"content one"),
            ("file2.txt", b"content two"),
        ]);
        let resp = do_request(body, ct).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        // FAILS on broken fixture: count is 1, not 2
        assert_eq!(json["count"], 2, "BUG-002: Backend only processes first file");
        assert_eq!(json["uploaded"].as_array().unwrap().len(), 2);
    }

    /// TEST 03: Upload three valid files — also fails due to BUG-002
    #[tokio::test]
    async fn test_upload_three_valid_files_bug002() {
        let (ct, body) = build_multipart_correct(&[
            ("a.txt", b"aaa"),
            ("b.txt", b"bbb"),
            ("c.png", &[0u8; 100]),
        ]);
        let resp = do_request(body, ct).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        // FAILS on broken fixture
        assert_eq!(json["count"], 3, "BUG-002: Only first file processed");
    }

    /// TEST 04: Correct multipart field name "files" must be accepted
    #[tokio::test]
    async fn test_correct_field_name_accepted() {
        let (ct, body) = build_multipart_correct(&[("test.txt", b"data")]);
        let resp = do_request(body, ct).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    /// TEST 05: Empty upload request — should return 400
    #[tokio::test]
    async fn test_empty_upload_rejected() {
        let boundary = "----TestBoundary12345";
        let body = format!("--{}--\r\n", boundary).into_bytes();
        let ct = format!("multipart/form-data; boundary={}", boundary);
        let resp = do_request(body, ct).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    /// TEST 06: Unsupported file format — should return 422
    #[tokio::test]
    async fn test_unsupported_format_rejected() {
        let (ct, body) = build_multipart_correct(&[("malware.exe", b"MZ\x90\x00")]);
        let resp = do_request(body, ct).await;
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    /// TEST 07: File exceeding 5 MB limit — should return 422
    #[tokio::test]
    async fn test_file_size_limit_enforced() {
        let big = vec![0u8; 5 * 1024 * 1024 + 1];
        let (ct, body) = build_multipart_correct(&[("large.txt", &big)]);
        let resp = do_request(body, ct).await;
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    /// TEST 08: Response includes correct count
    #[tokio::test]
    async fn test_response_count_matches_files() {
        let (ct, body) = build_multipart_correct(&[("only.txt", b"x")]);
        let resp = do_request(body, ct).await;
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["count"].as_u64().unwrap(), json["uploaded"].as_array().unwrap().len() as u64);
    }

    /// TEST 09: Successful response has required schema fields
    #[tokio::test]
    async fn test_response_schema() {
        let (ct, body) = build_multipart_correct(&[("schema_test.txt", b"hello")]);
        let resp = do_request(body, ct).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(json.get("uploaded").is_some(), "Missing 'uploaded' field");
        assert!(json.get("count").is_some(), "Missing 'count' field");
        assert!(json["uploaded"][0].get("filename").is_some());
        assert!(json["uploaded"][0].get("size_bytes").is_some());
        assert!(json["uploaded"][0].get("status").is_some());
    }

    /// TEST 10: No silent omission — all accepted files must appear in response
    #[tokio::test]
    async fn test_no_silent_omission_bug002() {
        let (ct, body) = build_multipart_correct(&[
            ("first.txt", b"first content"),
            ("second.txt", b"second content"),
        ]);
        let resp = do_request(body, ct).await;
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let filenames: Vec<&str> = json["uploaded"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["filename"].as_str().unwrap())
            .collect();
        // BUG-002: "second.txt" will be missing
        assert!(filenames.contains(&"second.txt"), "BUG-002: second.txt silently omitted");
    }

    /// TEST 11: Rejection of invalid batch
    /// On the broken fixture, BUG-002 causes the backend to only check the FIRST file.
    /// If the first file is valid, the backend returns 200 with count=1 (ignoring the invalid second file).
    /// The correct behavior should reject the entire batch when any file is invalid.
    /// This test verifies the BROKEN behavior — it should fail after the fix.
    #[tokio::test]
    async fn test_invalid_batch_behavior_bug002() {
        let (ct, body) = build_multipart_correct(&[
            ("valid.txt", b"ok"),
            ("bad.pdf", b"PDF content"),
        ]);
        let resp = do_request(body, ct).await;
        // BUG-002: The broken backend only processes first file (valid.txt) and returns 200
        // This is wrong — the batch should be rejected because bad.pdf is invalid
        // After fix, this should return 422
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        // On broken fixture: count=1 (only first file processed, second ignored)
        // BUG-002 means we never detect the invalid second file
        assert_eq!(json["count"], 1, "BUG-002: Backend only processes first file, missing invalid second file detection");
    }
}
