use crate::{error::AppError, models::{auth::{Claims, User}, self}};
use std::sync::Arc;

use axum::{Json, extract::State};
use jsonwebtoken::{encode, Header};
use serde_json::{json, Value};
use sqlx::PgPool;
use tracing::{info, warn};

use crate::{
    utils::get_timestamp_8_hours_from_now,
    AppState, KEYS,
};

pub async fn user_profile(claims: Claims) -> Result<axum::Json<serde_json::Value>, AppError> {
    // if the token is verified and data is extracted from the token by the implimentation in utils.rs then only the below code will run
    Ok(axum::Json(serde_json::json!({"username": claims.username})))
}


pub async fn find_user(pool: &PgPool, user_id: &str) -> Result<User, AppError> {
    // get the user for the email from database
    let user = sqlx::query_as::<_, models::auth::User>(
        r#"
        SELECT email, password, name, is_admin FROM users
        where email = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|err| {
        dbg!(err);
        AppError::InternalServerError
    })?;

    user.ok_or(AppError::UserDoesNotExist)
}


pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(credentials): Json<models::auth::User>,
) -> Result<Json<Value>, AppError> {
    if credentials.email.is_empty() {
        return Err(AppError::MissingCredential);
    }

    let user = find_user(&state.pool, &credentials.email).await?;

    let matches = argon2::verify_encoded(&user.password, &credentials.password.as_bytes()).unwrap();

    if matches {}
    
    if matches {
        info!("User: {} successfully logged in", &credentials.email);
        let claims = Claims {
            username: user.email,
            is_admin: user.is_admin,
            exp: get_timestamp_8_hours_from_now(),
        };
        let token = encode(&Header::default(), &claims, &KEYS.encoding)
        .map_err(|_| AppError::TokenCreation)?;
        // return bearer token
        Ok(Json(json!({ "access_token": token, "type": "Bearer" })))
    } else {
        warn!("Wrong credentials for user: {}", &credentials.email);
        Err(AppError::WrongCredential)
    }
}
