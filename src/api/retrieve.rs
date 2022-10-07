use std::sync::Arc;

use axum::{extract::Query, Extension};
use serde::Deserialize;
use serde_json::json;

use crate::{
    error::AppError,
    models::{auth::Claims, results::Results},
    AppState,
};

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
    log::info!("get_results user:{} query: {:?}", claims.username, query);

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
