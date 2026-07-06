use anyhow::Result;
use axum::Router;
use clap::Parser;
use conduit::config::Config;
use conduit::state::AppState;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Load env variables from `.env` if present.
    dotenvy::dotenv().ok();

    // Initialize the logger.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Extract configuration from CLI arguments or env variables.
    let config = Config::parse();

    // Establish the PostgreSQL connection pool.
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.database_url)
        .await?;

    // Apply pending migrations before starting the application.
    sqlx::migrate!().run(&pool).await?;

    let state = AppState {
        db: pool,
        jwt_secret: config.jwt_secret.clone(),
    };
    let app = Router::new()
        .nest(
            "/api",
            conduit::users::router()
                .merge(conduit::profiles::router().merge(conduit::articles::router())),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&config.host).await?;
    tracing::info!("listening on {}", config.host);
    axum::serve(listener, app).await?;

    Ok(())
}
