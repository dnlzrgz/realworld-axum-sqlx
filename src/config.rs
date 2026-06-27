use std::net::SocketAddr;

use clap::Parser;

// Config parameters for the application.
#[derive(Parser)]
#[command(version, about="", long_about=None)]
pub struct Config {
    /// Connection URL for the PostgreSQL database.
    #[arg(long, env)]
    pub database_url: String,

    /// Maximum number of connections in the pool.
    ///
    /// At least must be 1 and should not exceed the database's
    /// connection limit.
    #[arg(long, env, default_value_t=10, value_parser = clap::value_parser!(u32).range(1..))]
    pub max_connections: u32,

    /// Secret key to sign and verify JWT tokens.
    #[arg(long, env)]
    pub jwt_secret: String,

    /// Host and port to bind the HTTP server on.
    #[arg(long, env, default_value = "0.0.0.0:3000")]
    pub host: SocketAddr,
}

// Custom Debug implementation to avoid leaking secrets.
impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("database_url", &self.database_url)
            .field("max_connections", &self.max_connections)
            .field("jwt_secret", &"[REDACTED]")
            .field("host", &self.host)
            .finish()
    }
}
