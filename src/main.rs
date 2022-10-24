use axum::{
    extract::Extension,
    routing::{get, post},
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use cergdb::{
    api::{
        self,
        auth::{find_user, insert_new_user},
    },
    models::auth::User,
    AppState, MIGRATOR,
};
use secrecy::Secret;
use sqlx::postgres::PgPoolOptions;
use std::{env, fs, path::PathBuf, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    let root_path = env::var("CARGO_MANIFEST_DIR")
        .map_or(env::current_dir().unwrap_or(PathBuf::from("")), |x| {
            PathBuf::from(x)
        });
    dotenv::from_filename(root_path.join(".env")).ok();

    // pretty_env_logger::init();

    let db_user = env::var("DB_USER").unwrap();
    let db_password = env::var("DB_PASSWORD").unwrap_or_default();
    let db_host = env::var("DB_HOST").unwrap();
    let db_port = env::var("DB_PORT").unwrap();
    let db_name = env::var("DB_NAME").unwrap();
    let db_url = format!("postgres://{db_user}:{db_password}@{db_host}:{db_port}/{db_name}");

    // initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "axum=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cors = CorsLayer::new().allow_origin(Any);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("unable to connect to database");

    let secret = Secret::from(env::var("SECRET").unwrap_or(
        fs::read_to_string(root_path.join("SECRET")).expect("Could not open SECRET file."),
    ));

    MIGRATOR.run(&pool).await.unwrap();

    let state = Arc::new(AppState { pool, secret });

    if find_user(&state.pool, "admin").await.is_err() {
        let password = env::var("ADMIN_PASSWORD").unwrap_or(
            fs::read_to_string(root_path.join("PASSWORD")).expect("Could not open PASSWORD file."),
        );
        log::info!("setting admin password");
        let admin = User {
            email: "admin".to_owned(),
            password: password,
            name: "Administrator".to_owned(),
            is_admin: true,
        };
        let mut transaction = state.pool.begin().await.unwrap();
        let new_user_id = insert_new_user(&state, &mut transaction, &admin)
            .await
            .unwrap();
        transaction.commit().await.unwrap();
        assert!(new_user_id == "admin");
    }

    let app = Router::new()
        .route("/", get(api::info::route_info))
        .route("/login", post(api::auth::login))
        .route("/register", post(api::auth::register))
        //only logged-in user can access this route
        .route("/user_profile", get(api::users::user_profile))
        .route("/submit", post(api::submit))
        .route("/delete", post(api::delete))
        .route("/retrieve", post(api::retrieve))
        .layer(cors)
        .layer(Extension(state));

    let ip = env::var("SERVER_IP")
        .unwrap_or("0.0.0.0".to_string())
        .parse()
        .unwrap();
    let port = env::var("SERVER_PORT").unwrap().parse().unwrap();
    let tls = env::var("TLS").unwrap().parse().unwrap();

    log::info!(
        "Starting server at {}://{}:{}",
        if tls { "https" } else { "http" },
        ip,
        port
    );

    let addr = std::net::SocketAddr::new(ip, port);
    tracing::debug!("listening on {}", &addr);
    // let server = axum::Server::bind(&addr);
    if tls {
        let config = RustlsConfig::from_pem_file(
            PathBuf::from(env::var("TLS_CERT_PEM").unwrap()),
            PathBuf::from(env::var("TLS_KEY_PEM").unwrap()),
        )
        .await
        .unwrap();

        axum_server::bind_rustls(addr, config)
            .serve(app.into_make_service())
            .await
            .expect("failed to start server");
    } else {
        axum_server::bind(addr)
            .serve(app.into_make_service())
            .await
            .expect("failed to start server");
    };
}
