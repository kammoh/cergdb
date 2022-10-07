use jsonwebtoken::{DecodingKey, EncodingKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize)]
pub struct User {
    pub email: String,
    #[serde(skip_serializing)]
    pub password:String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub is_admin: bool,
}

#[derive(Deserialize, Serialize)]
pub struct Claims {
    pub username: String,
    pub is_admin: bool,
    pub exp: u64,
}

pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl Keys {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}
