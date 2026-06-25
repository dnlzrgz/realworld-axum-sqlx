use clap::Parser;
use conduit::config::Config;

fn main() {
    dotenvy::dotenv().ok();

    let config = Config::parse();
    println!("{:?}", config);
}
