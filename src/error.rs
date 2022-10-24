use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    WrongCredential,
    MissingCredential,
    TokenCreation,
    InternalServerError,
    InvalidQuery,
    UserDoesNotExist,
    UserAlreadyExits,
    ResultsAlreadyExits,
    ResultsNotFound(String),
    AuthenticationError(String),
    TokenError(String),
    SqlxError(sqlx::Error),
}

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
            Self::UserAlreadyExits => (StatusCode::BAD_REQUEST, "User already exists".into()),
            Self::InvalidQuery => (StatusCode::BAD_REQUEST, "Invalid query".into()),
            Self::ResultsNotFound(id) => (
                StatusCode::BAD_REQUEST,
                format!("No results with id={id} were found."),
            ),
            Self::ResultsAlreadyExits => (
                StatusCode::BAD_REQUEST,
                "Results already exist. Try update".into(),
            ),
            Self::SqlxError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {e}"),
            ),
        };
        (status, Json(json!({ "error": err_msg }))).into_response()
    }
}
