#![allow(unused)]

use anyhow::bail;
use futures::StreamExt;
use futures::sink::SinkExt;
use futures::stream::SplitSink;
use std::fmt;
use std::fmt::Formatter;
use std::net::SocketAddr;
use std::os::macos::raw::stat;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio_util::codec::{Framed, LinesCodec, LinesCodecError};
use tracing::{info, warn};
use tracing_subscriber::Layer as _;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Debug, Default)]
struct State {
    value: dashmap::DashMap<SocketAddr, Sender<Arc<Message>>>,
}

impl State {
    async fn broadcast(&self, addr: SocketAddr, message: Arc<Message>) -> anyhow::Result<()> {
        for entry in self.value.iter() {
            let key = entry.key();
            if key != &addr {
                let value = entry.value();
                let send_msg = message.clone();
                if let Err(e) = value.send(send_msg).await {
                    warn!("failed to send message,need remove: {:#?}", e);
                    self.value.remove(&addr);
                };
            }
        }
        anyhow::Ok(())
    }

    fn add(&self, addr: SocketAddr, mut peer: Peer) -> anyhow::Result<()> {
        // self.value.insert(addr, sender);
        let (sc, mut rc) = mpsc::channel(128);
        self.value.insert(addr, sc);
        tokio::spawn(async move {
            loop {
                match rc.recv().await {
                    None => {
                        info!("peer {:?} disconnected", addr);
                        break;
                    }
                    Some(value) => {
                        if let Err(e) = peer.sender.send(value.to_string()).await {
                            warn!("Failed to send message: {:#?}", e);
                            break;
                        };
                    }
                }
            }
        });
        anyhow::Ok(())
    }
}

#[derive(Debug)]
enum Message {
    UserLogin(String),
    UserOut(String),
    Chat(String, String),
}

impl Message {
    fn new_user_login(username: impl Into<String>) -> Self {
        Message::UserLogin(username.into())
    }

    fn new_user_logout(username: impl Into<String>) -> Self {
        Message::UserOut(username.into())
    }

    fn new_chat(username: impl Into<String>, msg: impl Into<String>) -> Self {
        Message::Chat(username.into(), msg.into())
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Message::UserLogin(username) => write!(f, "[{username} login ]"),
            Message::UserOut(username) => write!(f, "[ {username} logout ]"),
            Message::Chat(username, info) => write!(f, "{username}: {info}"),
        }
    }
}

struct Peer {
    username: String,
    sender: SplitSink<Framed<TcpStream, LinesCodec>, String>,
}
// use tokio_console;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let layer = Layer::new().pretty().with_filter(LevelFilter::INFO);
    let console_layer = console_subscriber::spawn();
    tracing_subscriber::registry()
        .with(layer)
        .with(console_layer)
        .init();
    let addr = "0.0.0.0:7878";
    info!("listening on {}", addr);
    let stream = TcpListener::bind(addr).await?;
    let state = Arc::new(State::default());
    loop {
        let (stream, self_addr) = stream.accept().await?;
        info!("accepted a new connection from {}", self_addr);
        let clone_state = Arc::clone(&state);
        tokio::spawn(async move {
            // start
            let peer = match handle_connect(stream, self_addr, clone_state.clone()).await {
                Ok(peer) => peer,
                Err(e) => {
                    warn!("connect error: {:#?}", e);
                    bail!("error")
                }
            };
            clone_state.add(self_addr, peer)?;
            anyhow::Ok(())
        });
    }
}

async fn handle_connect(
    stream: TcpStream,
    addr: SocketAddr,
    state: Arc<State>,
) -> anyhow::Result<Peer> {
    // 封装为framed
    let mut stream = Framed::new(stream, LinesCodec::new());
    let (mut writer, mut receiver) = stream.split();
    writer.send("pls send your name".to_string()).await?;
    let username = match receiver.next().await {
        None => {
            bail!("stream ended unexpectedly")
        }
        Some(msg) => match msg {
            Ok(username) => username,
            Err(e) => bail!(e),
        },
    };
    info!("username join: {}", username);
    state
        .broadcast(addr, Arc::new(Message::new_user_login(username.clone())))
        .await?;
    let new_state = state.clone();
    let new_user = username.clone();
    tokio::spawn(async move {
        while let Some(Ok(line)) = receiver.next().await {
            if let Err(e) = new_state
                .broadcast(addr, Arc::new(Message::new_chat(new_user.clone(), line)))
                .await
            {
                warn!("failed to broadcast {:?}", e);
                break;
            };
        }
        state.value.remove(&addr);
        anyhow::Ok(())
    });

    anyhow::Ok(Peer {
        username,
        sender: writer,
    })
}
