use axum::{extract::State, Json};
use serde_json::json;

use crate::errors::ApiResult;
use crate::state::SharedState;

pub async fn list_projects(
    State(_state): State<SharedState>,
) -> ApiResult<Json<serde_json::Value>> {
    // For MVP: return the single bundled UploadLab project
    Ok(Json(json!({
        "projects": [
            {
                "id": "uploadlab",
                "name": "UploadLab",
                "description": "Multi-file upload application with deliberate API contract defects",
                "path": "sample_project/broken"
            }
        ]
    })))
}

pub async fn get_project(
    State(_state): State<SharedState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    if id != "uploadlab" {
        return Err(crate::errors::AppError::NotFound(format!(
            "Project {} not found",
            id
        )));
    }
    Ok(Json(json!({
        "project": {
            "id": "uploadlab",
            "name": "UploadLab",
            "description": "Multi-file upload application with deliberate API contract defects",
            "path": "sample_project/broken"
        }
    })))
}

pub async fn list_scenarios(
    State(_state): State<SharedState>,
    axum::extract::Path(project_id): axum::extract::Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    if project_id != "uploadlab" {
        return Err(crate::errors::AppError::NotFound(format!(
            "Project {} not found",
            project_id
        )));
    }
    Ok(Json(json!({
        "scenarios": [
            {
                "id": "multi-file-upload-failure",
                "project_id": "uploadlab",
                "name": "Multi-File Upload Failure",
                "description": "Frontend sends wrong field name (BUG-001) and backend processes only first file (BUG-002)",
                "bug_report": "POST /api/upload fails when multiple files are selected. Only one file appears in the response and the field name used by the frontend does not match the documented API contract."
            }
        ]
    })))
}
