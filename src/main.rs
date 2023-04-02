use axum::{
    error_handling::HandleErrorLayer,
    routing::{get, post},
    Router, http::StatusCode, extract::DefaultBodyLimit,
};
use axum_server::tls_rustls::RustlsConfig;
use cergdb::{
    api::{self, admin::insert_new_user, users::find_user},
    models::auth::User,
    AppState, MIGRATOR,
};
use clap::Parser;
use miette::IntoDiagnostic;
use secrecy::Secret;
use sqlx::postgres::PgPoolOptions;
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::{env, fs, path::PathBuf, sync::Arc, time::Duration};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, value_name = "DIR")]
    config_dir: Option<PathBuf>,

    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[arg(short, long, value_name = "FILE")]
    secret: Option<PathBuf>,

    #[arg(short, long, value_name = "FILE")]
    password: Option<PathBuf>,

    #[arg(short, long, value_name = "DIR")]
    logs_dir: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    let args = Args::parse();

    let root_path = args.config_dir.unwrap_or(
        env::var("CARGO_MANIFEST_DIR")
            .map_or(env::current_dir().unwrap_or(PathBuf::from("")), |x| {
                PathBuf::from(x)
            }),
    );
    dotenv::from_filename(args.config.unwrap_or(root_path.join(".env"))).ok();

    let db_user = env::var("DB_USER").into_diagnostic()?;
    let db_password = env::var("DB_PASSWORD").unwrap_or_default();
    let db_host = env::var("DB_HOST").into_diagnostic()?;
    let db_port = env::var("DB_PORT").into_diagnostic()?;
    let db_name = env::var("DB_NAME").into_diagnostic()?;
    let db_url = format!("postgres://{db_user}:{db_password}@{db_host}:{db_port}/{db_name}");

    // initialize tracing

    let file_appender =
        tracing_appender::rolling::daily(args.logs_dir.unwrap_or(root_path.join("logs")), "cergdb");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "cergdb=debug,axum=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer().pretty())
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
        .init();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("unable to connect to database");

    let secret = Secret::from(
        env::var("SECRET").unwrap_or(
            fs::read_to_string(args.secret.unwrap_or(root_path.join("SECRET")))
                .expect("Could not open SECRET file."),
        ),
    );

    MIGRATOR.run(&pool).await.into_diagnostic()?;

    let state = Arc::new(AppState { pool, secret });

    if find_user(&state.pool, "admin").await.is_err() {
        let password = env::var("ADMIN_PASSWORD").unwrap_or(
            fs::read_to_string(args.password.unwrap_or(root_path.join("PASSWORD")))
                .expect("Could not open PASSWORD file."),
        );
        log::info!("setting admin password");
        let admin = User {
            email: "admin".to_owned(),
            password: password,
            name: "Administrator".to_owned(),
            is_admin: true,
        };
        let mut transaction = state.pool.begin().await.into_diagnostic()?;
        let new_user_id = insert_new_user(&state, &mut transaction, &admin)
            .await
            .into_diagnostic()?;
        transaction.commit().await.into_diagnostic()?;
        assert!(new_user_id == "admin");
    }

    let app = Router::new()
        .route("/", get(api::info::route_info))
        .route("/login", post(api::users::login))
        .route("/register", post(api::admin::register))
        .route("/user_profile", get(api::users::user_profile))
        .route("/submit", post(api::submit::submit))
        .route("/delete", post(api::delete))
        .route("/retrieve", post(api::retrieve))
        .route("/rename", post(api::rename))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<tower::timeout::error::Elapsed>() {
                        Ok(StatusCode::REQUEST_TIMEOUT)
                    } else {
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled internal error: {}", error),
                        ))
                    }
                }))
                .timeout(Duration::from_secs(10))
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        )
        .layer(DefaultBodyLimit::disable())
        .with_state(state);

    let ip = env::var("SERVER_IP")
        .unwrap_or("0.0.0.0".to_string())
        .parse()
        .into_diagnostic()?;
    let port = env::var("SERVER_PORT")
        .into_diagnostic()?
        .parse()
        .into_diagnostic()?;
    let tls = env::var("TLS")
        .into_diagnostic()?
        .parse()
        .into_diagnostic()?;

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
            PathBuf::from(env::var("TLS_CERT_PEM").into_diagnostic()?),
            PathBuf::from(env::var("TLS_KEY_PEM").into_diagnostic()?),
        )
        .await
        .into_diagnostic()?;

        axum_server::bind_rustls(addr, config)
            .serve(app.into_make_service())
            .await
            .expect("failed to start TLS server");
    } else {
        axum_server::bind(addr)
            .serve(app.into_make_service())
            .await
            .expect("failed to start server");
    };
    Ok(())
}
