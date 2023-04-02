use std::sync::Arc;

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{error::AppError, models::auth::Claims, AppState};

#[derive(Serialize, Deserialize)]
pub struct RenameRequest {
    current_id: String,
    new_id: String,
}

pub async fn rename(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Json(request): Json<RenameRequest>,
) -> Result<axum::Json<serde_json::Value>, AppError> {
    let mut transaction = state.pool.begin().await?;

    if sqlx::query!(
        r#"SELECT * from results WHERE id = $1;"#,
        &request.current_id,
    )
    .fetch_optional(&mut transaction)
    .await?
    .is_none()
    {
        return Err(AppError::IdNotFound(request.current_id.clone()));
    }

    if sqlx::query!(r#"SELECT id from results WHERE id = $1;"#, &request.new_id,)
        .fetch_optional(&mut transaction)
        .await?
        .is_some()
    {
        return Err(AppError::IdExists(request.new_id.clone()));
    }

    sqlx::query!(
        r#"UPDATE results SET id=$1 WHERE id=$2;"#,
        request.new_id,
        request.current_id,
    )
    .execute(&mut transaction)
    .await?;

    transaction.commit().await?;

    Ok(axum::Json(json!({
        "success": true,
        "old_id": request.current_id,
        "new_id" : request.new_id,
        "submitter": claims.username,
    })))
}
