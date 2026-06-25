use clap::Parser;

// Config parameters for the application.
#[derive(Parser, Debug)]
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
}
