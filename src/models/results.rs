use serde::{Deserialize, Serialize};
use sqlx::types::JsonValue;

#[derive(Deserialize, Serialize, PartialEq, Debug, sqlx::FromRow)]
pub struct Results {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub metadata: JsonValue,
    #[serde(default)]
    pub timing: JsonValue,
    #[serde(default)]
    pub synthesis: JsonValue,
}
