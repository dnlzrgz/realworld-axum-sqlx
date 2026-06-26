use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use conduit::config::Config;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<()> {
    // Load env variables from `.env` if present.
    dotenvy::dotenv().ok();

    // Extract configuration from CLI arguments or env variables.
    let config = Config::parse();

    // Establish the PostgreSQL connection pool.
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.database_url)
        .await
        .context("failed to establish PostgreSQL connection pool")?;

    // Apply pending migratins before starting the application.
    sqlx::migrate!().run(&pool).await?;

    Ok(())
}
