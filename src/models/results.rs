use serde::{Deserialize, Serialize};
use sqlx::types::JsonValue;

#[derive(Deserialize, Serialize, PartialEq, Debug, sqlx::FromRow)]
pub struct Results {
    pub id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[sqlx(default)]
    pub name: String,
    #[sqlx(default)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
