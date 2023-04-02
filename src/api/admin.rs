use std::sync::Arc;

use axum::{Json, extract::State};
use secrecy::ExposeSecret;
use serde_json::{json, Value};
use sqlx::{PgPool, Postgres, Transaction};
use crate::{
    error::AppError,
    models::{
        self,
        auth::{Claims, User},
    }, AppState,
};

pub async fn insert_new_user(
    state: &AppState,
    transaction: &mut Transaction<'_, Postgres>,
    new_user: &User,
) -> Result<String, AppError> {
    let argon2_config = argon2::Config::default();
    let password_hash = argon2::hash_encoded(
        new_user.password.as_bytes(),
        state.secret.expose_secret().as_bytes(),
        &argon2_config,
    )
    .unwrap();

    match sqlx::query!(
        r#"
        INSERT INTO users (
            email,
            password,
            name,
            is_admin
        )
        VALUES ($1, $2, $3, $4)
        ON CONFLICT DO NOTHING
        RETURNING email
        "#,
        new_user.email,
        password_hash,
        new_user.name,
        new_user.is_admin,
    )
    .fetch_optional(transaction)
    .await
    {
        Ok(Some(record)) => Ok(record.email),
        Ok(None) => Err(AppError::UserAlreadyExits),
        Err(err) => Err(AppError::SqlxError(err)),
    }
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Json(new_user): Json<models::auth::User>,
) -> Result<Json<Value>, AppError> {
    if !claims.is_admin {
        return Err(AppError::AuthenticationError(format!(
            "User {} does not have admin privileges",
            claims.username
        )));
    }

    // check if email or password is a blank string
    if new_user.email.is_empty() || new_user.password.is_empty() {
        return Err(AppError::MissingCredential);
    }

    let mut transaction = state.pool.begin().await?;
    let new_user_id = insert_new_user(&state, &mut transaction, &new_user).await?;
    transaction.commit().await?;

    Ok(Json(json!({
        "success": format!("registered user: {new_user_id}")
    })))
}
