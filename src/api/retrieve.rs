use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use axum::{extract::State, Json};
use flatten_json_object::Flattener;
use indexmap::IndexSet;
use json_dotpath::DotPaths;
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
    fields: Option<Vec<String>>,
    #[serde(default)]
    flatten: bool,
}

fn do_filter(field: &mut serde_json::Value, field_name: &str, fields: &Vec<String>) {
    if fields.contains(&field_name.to_owned()) {
        return;
    }
    let paths = get_sub_paths(field_name, fields);
    let mut key_values: HashMap<_, serde_json::Value> = HashMap::new();
    for k in paths {
        if let Some(v) = field.dot_get(k).unwrap_or_default() {
            key_values.insert(k, v);
        }
    }
    *field = json!(key_values);
}

fn get_sub_paths<'a>(field_name: &'a str, fields: &'a Vec<String>) -> Vec<&'a str> {
    fields
        .iter()
        .filter_map(|s| {
            s.split_once(".")
                .and_then(|tup| (tup.0 == field_name).then_some(tup.1))
        })
        .collect()
}

pub async fn retrieve(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Json(query): Json<GetResultsParams>,
) -> Result<axum::Json<serde_json::Value>, AppError> {
    log::info!("get_results user:{} query: {:?}", claims.username, query);

    // let filter = query.filter.map_or(String::new(), |s| {
    //     // FIXME FIXME securely verify filter, exploitable serious security flaw!!!
    //     format!("WHERE {}", s)
    // });

    let filter = "";

    let json_fields = HashSet::from(["metadata", "timing", "synthesis"]);

    let field_sel = if let Some(ref fields) = query.fields {
        let mut field_names: IndexSet<_> = IndexSet::new();
        field_names.insert("id");
        field_names.extend(fields.iter().filter_map(|s| {
            match s.as_str() {
                "id" => None,
                "name" | "category" => Some(s.as_str()),
                _ => s
                    .split(".")
                    .next()
                    .filter(|first| json_fields.contains(first)),
            }
        }));
        field_names.into_iter().collect::<Vec<_>>().join(", ")
    } else {
        "*".to_owned()
    };

    let sql = format!(
        r#"SELECT {} from results
        {filter}
        ORDER BY id ASC
        OFFSET {}
        LIMIT {};"#,
        field_sel,
        query.offset.unwrap_or(0),
        query.limit.map_or("ALL".to_owned(), |i| i.to_string()),
    );

    log::info!("sql={}", sql);

    let mut rows: Vec<Results> = sqlx::query_as(sql.as_str()).fetch_all(&state.pool).await?;

    if let Some(ref fields) = query.fields {
        for row in &mut rows {
            do_filter(&mut row.timing, "timing", fields);
            do_filter(&mut row.synthesis, "synthesis", fields);
            do_filter(&mut row.metadata, "metadata", fields);
        }
    };

    if query.flatten {
        let flattener = Flattener::new();

        for row in &mut rows {
            row.metadata = flattener.flatten(&row.metadata).unwrap_or_default();
            row.timing = flattener.flatten(&row.timing).unwrap_or_default();
            row.synthesis = flattener.flatten(&row.synthesis).unwrap_or_default();
        }
    }

    Ok(axum::Json(json!(rows)))
}
