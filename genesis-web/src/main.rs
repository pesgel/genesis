use clap::Parser;
use genesis_web::config::init_shared_app_state;
use genesis_web::{adapter, cmd::*, config};

#[tokio::main]
async fn main() {
    // step1. parse cli
    let cli = GenesisCli::parse();
    match cli.command {
        Commands::Run { config } => {
            // set config
            let config = config::parse_config(&config).await.unwrap();
            // error level
            let mut filter = tracing_subscriber::EnvFilter::from_default_env();
            // convert config
            if let Some(tracing) = &config.tracing {
                for x in tracing.filter.split(",") {
                    filter = filter.add_directive(x.parse().unwrap());
                }
            }
            // register
            tracing_subscriber::fmt().with_env_filter(filter).init();
            // init state
            let state = init_shared_app_state(&config).await.unwrap();
            // step2. start web
            adapter::http::server::start_http_server(&config, state)
                .await
                .unwrap();
        }
    }
}
