use std::env;

use clap::Parser;
use genesis_web::{adapter, cmd::*, config};
#[tokio::main]
async fn main() {
    // step1. parse cli
    let cli = GenesisCli::parse();
    match cli.command {
        Commands::Run { mode, config } => {
            // set log level
            env::set_var("RUST_LOG", mode.as_ref());
            tracing_subscriber::fmt::init();
            // set config
            config::parse_config(&config).await.unwrap();
        }
    }
    // step2. start web
    adapter::http::server::start_http_server().await.unwrap();
}
