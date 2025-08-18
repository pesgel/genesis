use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, warn};
use tracing_subscriber::Layer;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

struct Config {
    bind_address: String,
    out_address: String,
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let layer = fmt::Layer::new().pretty().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    let config = Arc::new(Config {
        bind_address: "0.0.0.0:8081".to_string(),
        out_address: "127.0.0.1:9099".to_string(),
    });
    // start listen
    let listener = TcpListener::bind(&config.bind_address).await?;
    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Accept connection from {}", addr);
        let clone_config = Arc::clone(&config);
        // start proxy
        tokio::spawn(async move {
            proxy(stream, clone_config.out_address.to_string()).await?;
            Ok::<_, anyhow::Error>(())
        });
    }
}
async fn proxy(mut in_io: TcpStream, out_addr: String) -> anyhow::Result<()> {
    let mut out_stream = TcpStream::connect(out_addr).await?;
    let (mut out_reader, mut out_writer) = out_stream.split();
    let (mut in_reader, mut in_writer) = in_io.split();
    if let Err(e) = tokio::try_join!(
        tokio::io::copy(&mut in_reader, &mut out_writer),
        tokio::io::copy(&mut out_reader, &mut in_writer),
    ) {
        warn!("Failed to proxy output stream: {}", e);
    }
    anyhow::Ok(())
}
