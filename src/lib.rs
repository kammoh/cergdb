use once_cell::sync::Lazy;
use secrecy::Secret;
use sqlx::migrate::Migrator;

// import module
pub mod api;
pub mod error;
pub mod models;
pub mod utils;
pub mod http_server;

// secret key for JWT token
static KEYS: Lazy<models::auth::Keys> = Lazy::new(|| {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "Your secret here".to_owned());
    models::auth::Keys::new(secret.as_bytes())
});

static MIGRATOR: Migrator = sqlx::migrate!(); // defaults to "./migrations"

pub struct AppState {
    pub pool: sqlx::PgPool,
    pub secret: Secret<String>,
}
