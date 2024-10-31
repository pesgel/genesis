use core::str;

use axum::{
    body::Bytes,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};

use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use genesis_common::{EventHub, SshTargetPasswordAuth, TargetSSHOptions};
use genesis_ssh::start_ssh_connect;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;
use uuid::Uuid;

use crate::config::AppState;

/// start ssh
pub async fn handler_ssh(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    info!("hand ssh config: {:?}", state);
    // step1. 建立到远程服务的连接并且初始化事件处理器
    let uuid = Uuid::new_v4();
    let option = TargetSSHOptions {
        host: "10.0.1.52".into(),
        port: 22,
        username: "root".into(),
        allow_insecure_algos: Some(true),
        auth: genesis_common::SSHTargetAuth::Password(SshTargetPasswordAuth {
            password: "1qaz2wsx".into(),
        }),
    };
    let (hub, xs) = start_ssh_connect(uuid, option).await.unwrap();
    // step2. 绑定输入输出
    ws.on_upgrade(|socket| async move {
        let (sender, receiver) = socket.split();
        tokio::spawn(write(sender, hub));
        tokio::spawn(read(receiver, xs));
    })
}

async fn read(mut receiver: SplitStream<WebSocket>, sender: UnboundedSender<Bytes>) {
    tracing::info!("start ws receiver");
    while let Some(Ok(message)) = receiver.next().await {
        match message {
            Message::Text(text) => {
                let env: Envelope = serde_json::from_str(&text).unwrap();
                sender.send(Bytes::from(env.payload)).unwrap();
            }
            Message::Binary(bin) => {
                println!("Received binary data: {:?}", bin);
            }
            _ => {}
        }
    }
    tracing::info!("end ws receiver");
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Envelope {
    version: String,
    r#type: String,
    payload: String,
}

async fn write(mut sender: SplitSink<WebSocket, Message>, hub: EventHub<Bytes>) {
    let mut rec = hub.subscribe(|_| true).await;
    while let Some(bytes) = rec.recv().await {
        let data = str::from_utf8(&bytes).unwrap();
        // build
        let x = Envelope {
            version: "1.0".into(),
            r#type: "r".into(),
            payload: data.into(),
        };
        let st = serde_json::to_string(&x).unwrap();

        if let Err(e) = sender.send(Message::Text(st)).await {
            eprintln!("Error sending message: {}", e);
            break;
        }
    }
    tracing::info!("end ws writer");
}
