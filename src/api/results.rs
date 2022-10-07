use std::sync::Arc;

use axum::{extract::Query, Extension, Json};
use serde::Deserialize;
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
        "submitter": claims.user.email,
    })))
}

#[derive(Debug, Deserialize)]
pub struct GetResultsParams {
    limit: Option<i32>,
    offset: Option<i32>,
    filter: Option<String>,
}

pub async fn retrieve(
    Query(query): Query<GetResultsParams>,
    Extension(state): Extension<Arc<AppState>>,
    claims: Claims,
) -> Result<axum::Json<serde_json::Value>, AppError> {
    log::info!("get_results user:{} query: {:?}", claims.user.email, query);

    // let filter = query.filter.map_or(String::new(), |s| {
    //     // FIXME FIXME securely verify filter, exploitable serious security flaw!!!
    //     format!("WHERE {}", s)
    // });

    let filter = "";

    let rows: Vec<Results> = sqlx::query_as(
        format!(
            // r#"
            // SELECT * from results
            // {};"#,
            r#"
            SELECT * from results
            {filter}
            ORDER BY id ASC
            OFFSET {}
            LIMIT {}
            ;"#,
            query.offset.unwrap_or(0),
            query.limit.map_or("ALL".to_owned(), |i| i.to_string()),
        )
        .as_str(),
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(axum::Json(json!(rows)))
}
