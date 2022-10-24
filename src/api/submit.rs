use std::sync::Arc;

use axum::{Extension, Json};
use serde_json::json;
use sqlx::types::JsonValue;
use time::OffsetDateTime;

use crate::{
    error::AppError,
    models::{auth::Claims, results::Results},
    AppState,
};

pub async fn submit(
    Json(results): Json<Results>,
    Extension(state): Extension<Arc<AppState>>,
    claims: Claims,
) -> Result<axum::Json<serde_json::Value>, AppError> {
    let mut transaction = state.pool.begin().await?;

    let existing = sqlx::query!(
        r#"
        SELECT * from results
        WHERE id = $1
        "#,
        results.id,
    )
    .fetch_optional(&mut transaction)
    .await?;

    match existing {
        Some(record) => {
            log::info!("already exists, updating");

            let category = match results.category {
                None => record.category,
                _ => results.category,
            };

            let metadata = match results.metadata {
                JsonValue::Null => record.metadata,
                _ => Some(results.metadata),
            };
            let timing = match results.timing {
                JsonValue::Null => record.timing,
                _ => Some(results.timing),
            };
            let synthesis = match results.synthesis {
                JsonValue::Null => record.synthesis,
                _ => Some(results.synthesis),
            };
            sqlx::query!(
                r#"
            UPDATE results
            SET name      = $2,
                timestamp = $3,
                category  = $4,
                metadata  = $5,
                timing    = $6,
                synthesis = $7
            WHERE id = $1;
            "#,
                results.id,
                results.name,
                OffsetDateTime::now_utc(),
                category,
                metadata,
                timing,
                synthesis
            )
            .execute(&mut transaction)
            .await?
        }
        None => {
            sqlx::query!(
                r#"
                INSERT INTO results (
                    id,
                    name,
                    timestamp,
                    category,
                    metadata,
                    timing,
                    synthesis
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
                results.id,
                results.name,
                OffsetDateTime::now_utc(),
                results.category,
                results.metadata,
                results.timing,
                results.synthesis
            )
            .execute(&mut transaction)
            .await?
        }
    };

    transaction.commit().await?;

    Ok(axum::Json(json!({
        // "id": record.id,
        "submitter": claims.username,
    })))
}