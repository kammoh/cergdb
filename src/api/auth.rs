use std::sync::Arc;

use axum::{Extension, Json};
use jsonwebtoken::{encode, Header};
use secrecy::ExposeSecret;
use serde_json::{json, Value};
use sqlx::{PgPool, Postgres, Transaction};
use tracing::{info, warn};

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

pub async fn login(
    Json(credentials): Json<models::auth::User>,
    Extension(state): Extension<Arc<AppState>>,
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

pub async fn register(
    Json(new_user): Json<models::auth::User>,
    Extension(state): Extension<Arc<AppState>>,
    claims: Claims,
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
