use std::sync::Arc;

use axum::{Extension, Json};
use jsonwebtoken::{encode, Header};
use secrecy::ExposeSecret;
use serde_json::{json, Value};
use sqlx::{PgPool, Postgres, Transaction};

use crate::{
    error::AppError,
    models::{
        self,
        auth::{Claims, User},
    },
    utils::get_timestamp_8_hours_from_now,
    AppState, KEYS,
};

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

pub async fn insert_new_user(
    app_state: &AppState,
    transaction: &mut Transaction<'_, Postgres>,
    new_user: &User,
) -> Result<String, AppError> {
    let mut hasher = argonautica::Hasher::default();
    let password_hash = hasher
        .with_password(&new_user.password)
        .with_secret_key(app_state.secret.expose_secret())
        .hash()
        .expect("failed to hash the password");

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

pub async fn login(
    Json(credentials): Json<models::auth::User>,
    Extension(state): Extension<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    if credentials.email.is_empty() {
        return Err(AppError::MissingCredential);
    }

    let user = find_user(&state.pool, &credentials.email).await?;

    let verified = argonautica::Verifier::new()
        .with_secret_key(state.secret.expose_secret())
        .with_password(&credentials.password)
        .with_hash(&user.password)
        .verify();

    if verified.unwrap_or(false) {
        let claims = Claims {
            email: credentials.email.to_owned(),
            exp: get_timestamp_8_hours_from_now(),
        };
        let token = encode(&Header::default(), &claims, &KEYS.encoding)
            .map_err(|_| AppError::TokenCreation)?;
        // return bearer token
        Ok(Json(json!({ "access_token": token, "type": "Bearer" })))
    } else {
        Err(AppError::WrongCredential)
    }
}

pub async fn register(
    Json(new_user): Json<models::auth::User>,
    Extension(state): Extension<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
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
