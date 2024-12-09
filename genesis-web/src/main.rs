use std::env;

use clap::Parser;
use genesis_web::config::init_shared_app_state;
use genesis_web::{adapter, cmd::*, config};

#[tokio::main]
async fn main() {
    // step1. parse cli
    let cli = GenesisCli::parse();
    match cli.command {
        Commands::Run { mode, config } => {
            // set log level
            unsafe {
                env::set_var("RUST_LOG", mode.as_ref());
            }
            tracing_subscriber::fmt::init();
            // set config
            let config = config::parse_config(&config).await.unwrap();
            // init state
            let state = init_shared_app_state(&config).await.unwrap();
            // step2. start web
            adapter::http::server::start_http_server(&config, state)
                .await
                .unwrap();
        }
    }
}
