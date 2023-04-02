use serde::{Deserialize, Serialize};
use sqlx::types::JsonValue;

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Results {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[sqlx(default)]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[sqlx(default)]
    pub category: Option<String>,
    #[serde(default, skip_serializing_if = "JsonValue::is_null")]
    #[sqlx(default)]
    pub metadata: JsonValue,
    #[serde(default, skip_serializing_if = "JsonValue::is_null")]
    #[sqlx(default)]
    pub timing: JsonValue,
    #[serde(default, skip_serializing_if = "JsonValue::is_null")]
    #[sqlx(default)]
    pub synthesis: JsonValue,
}
