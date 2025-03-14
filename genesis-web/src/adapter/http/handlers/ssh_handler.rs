use core::str;
use std::time::Duration;

use axum::{
    body::{Body, Bytes},
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::StatusCode,
    response::Response,
};

use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use genesis_common::{EventHub, NotifyEnum, SshTargetPasswordAuth, TargetSSHOptions};
use genesis_ssh::start_ssh_connect;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::{common::Envelope, config::AppState};

/// start ssh
pub async fn handler_ssh(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    info!("hand ssh config: {:?}", state);
    // TODO 替换为真实获取登录数据的操作
    // step1. 建立到远程服务的连接并且初始化事件处理器
    let uuid = Uuid::new_v4();
    let option = TargetSSHOptions {
        host: "10.0.1.521".into(),
        port: 22,
        username: "root".into(),
        allow_insecure_algos: Some(true),
        auth: genesis_common::SSHTargetAuth::Password(SshTargetPasswordAuth {
            password: "1qaz2wsx".into(),
        }),
        pty_request: Default::default(),
    };
    match start_ssh_connect(uuid, option).await {
        Ok((hub, xs, mut abort_rc)) => {
            let (tx, rx) = unbounded_channel::<()>();
            // step2. 绑定输入输出
            ws.on_upgrade(move |socket| {
                let session_id = uuid;
                async move {
                    let (sender, receiver) = socket.split();
                    tokio::spawn(write(session_id, sender, hub, rx));
                    tokio::spawn(read(session_id, receiver, xs, tx.clone()));
                    tokio::spawn(async move {
                        match abort_rc.changed().await {
                            Ok(_) => match *abort_rc.borrow() {
                                NotifyEnum::SUCCESS => {}
                                _ => {
                                    info!("handler_ssh receive abort signal");
                                    let _ = tx.send(());
                                }
                            },
                            Err(e) => {
                                info!("handler_ssh receive abort signal error: {:?}", e);
                            }
                        };
                    });
                }
            })
        }
        Err(e) => {
            error!("create ssh connect error:{}", e);
            Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::empty())
                .unwrap()
        }
    }
}

async fn read(
    uuid: Uuid,
    mut receiver: SplitStream<WebSocket>,
    sender: UnboundedSender<Bytes>,
    close: UnboundedSender<()>,
) {
    debug!(session_id=%uuid, "start ws receiver");
    while let Some(Ok(message)) = receiver.next().await {
        match message {
            Message::Text(text) => match serde_json::from_str::<Envelope>(&text) {
                Ok(env) => {
                    if let Err(err) = sender.send(Bytes::from(env.payload)) {
                        error!(session_id=%uuid,"convert input to envelope error:{}",err);
                        break;
                    }
                }
                Err(err) => {
                    error!(session_id=%uuid,"serde deserialize error:{}",err);
                    break;
                }
            },
            Message::Binary(_) => {
                info!(session_id=%uuid,"binary type is not supported");
                break;
            }
            Message::Close(_) => {
                info!(session_id=%uuid,"client close ws");
                break;
            }
            //ping or pong not process
            _ => {}
        }
    }
    let _ = close.send(());
    info!(session_id=%uuid,"end ws receiver");
}

async fn write(
    uuid: Uuid,
    mut sender: SplitSink<WebSocket, Message>,
    hub: EventHub<Bytes>,
    mut receiver: UnboundedReceiver<()>,
) {
    let mut rec = hub.subscribe(|_| true).await;
    loop {
        tokio::select! {
            b = rec.recv() => match b{
                Some(bytes) => {
                    match str::from_utf8(&bytes) {
                        Ok(data) => {
                            let x = Envelope {
                                version: "1.0".into(),
                                r#type: "r".into(),
                                payload: data.into(),
                            };
                            match serde_json::to_string(&x) {
                                Ok(st) => {
                                    if let Err(e) = sender.send(Message::Text(st)).await {
                                        error!(session_id=%uuid,"error sending message:{}", e);
                                        break;
                                    }
                                },
                                Err(e) => {
                                    error!(session_id=%uuid,"serde to string error:{}", e);
                                    break;
                                },
                            };
                        },
                        Err(e) => {
                            error!(session_id=%uuid,"to utf8 string error:{}", e);
                            break;
                        },
                    }
                },
                None => {
                    info!(session_id=%uuid,"receive none")
                },
            },
            _ = receiver.recv() => {
                debug!(session_id=%uuid,"receive end");
                break
            }
            _ = tokio::time::sleep(Duration::from_secs(20)) => {
                debug!(session_id=%uuid,"time sleep end");
                break
            }
        }
    }
    let _ = sender.close().await;
    info!("end ws writer");
}
