use std::sync::Arc;

use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::query_as;

use crate::{
    error::AppError,
    models::{auth::Claims, results::Results},
    AppState,
};

#[derive(Serialize, Deserialize)]
pub struct DeleteRequest {
    id: String,
}

pub async fn delete(
    Json(request): Json<DeleteRequest>,
    Extension(state): Extension<Arc<AppState>>,
    claims: Claims,
) -> Result<axum::Json<serde_json::Value>, AppError> {
    let record: Option<Results> = query_as(r#"DELETE FROM results WHERE id = $1 RETURNING *"#)
        .bind(&request.id)
        .fetch_optional(&state.pool)
        .await?;
    match record {
        Some(deleted) => {
            assert!(request.id == deleted.id);
            log::info!("Deleted record with id={}", request.id);
            Ok(axum::Json(json!({
                "id": request.id,
                "deleted" : deleted,
                "user": claims.username,
            })))
        }
        None => Err(AppError::ResultsNotFound(request.id)),
    }
}
