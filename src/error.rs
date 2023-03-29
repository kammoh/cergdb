use std::fmt::Display;

use axum::{http::StatusCode, response::IntoResponse, Json};
use miette::Diagnostic;
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    WrongCredential,
    MissingCredential,
    TokenCreation,
    InternalServerError,
    InvalidQuery,
    UserDoesNotExist,
    UserAlreadyExits,
    IdNotFound(String),
    IdExists(String),
    AuthenticationError(String),
    InvalidToken,
    TokenError(String),
    SqlxError(sqlx::Error),
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Diagnostic for AppError {}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::SqlxError(e)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, err_msg) = match self {
            Self::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "an internal server error occurred".into(),
            ),
            Self::MissingCredential => (StatusCode::BAD_REQUEST, "missing credential".into()),
            Self::TokenCreation => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to create token".into(),
            ),
            Self::WrongCredential => (StatusCode::UNAUTHORIZED, "wrong credentials".into()),
            Self::UserDoesNotExist => (StatusCode::UNAUTHORIZED, "User does not exist".into()),
            Self::AuthenticationError(msg) => (
                StatusCode::UNAUTHORIZED,
                format!("Authentication error: {msg}"),
            ),
            Self::TokenError(msg) => (
                StatusCode::BAD_REQUEST,
                format!("Error in authentication token: {msg}"),
            ),
            Self::InvalidToken => (
                StatusCode::BAD_REQUEST,
                format!("Invalid authentication token"),
            ),
            Self::UserAlreadyExits => (StatusCode::BAD_REQUEST, "User already exists".into()),
            Self::InvalidQuery => (StatusCode::BAD_REQUEST, "Invalid query".into()),
            Self::IdExists(id) => (StatusCode::BAD_REQUEST, format!("ID: {id} already exist.")),
            Self::IdNotFound(id) => (StatusCode::BAD_REQUEST, format!("ID: {id} was not found.")),
            Self::SqlxError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {e}"),
            ),
        };
        (status, Json(json!({ "error": err_msg }))).into_response()
    }
}
