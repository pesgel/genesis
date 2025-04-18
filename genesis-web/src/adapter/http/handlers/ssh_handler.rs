use axum::{
    body::Bytes,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::Response,
};
use core::str;
use std::sync::Arc;

use crate::common::EnvelopeType;
use crate::{
    adapter::cmd::ssh::{ConnParams, SSHConnParams},
    common::{Envelope, SSHSessionCtx},
    config::{AppState, GLOBAL_MANAGER, SHARED_APP_CONFIG},
    error::AppError,
    repo::sea::NodeRepo,
};
use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use genesis_common::{PtyRequest, SshTargetPasswordAuth, TargetSSHOptions};
use genesis_process::{ExecuteState, SSHProcessManager};
use genesis_ssh::{ChannelOperation, ServerExtraEnum};
use tokio::sync::{broadcast, mpsc::UnboundedSender, watch, Mutex};
use tracing::{debug, error, info};
use uuid::Uuid;

pub async fn handler_ssh(
    ws: WebSocketUpgrade,
    Query(bq): Query<SSHConnParams>,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    // step1. 构造节点连接数据
    let query: ConnParams = serde_json::from_str(&bq.params)?;
    let node_en = NodeRepo::get_node_by_id(&state.conn, &query.opt_permission_id).await?;
    let uuid = Uuid::new_v4();
    let option = TargetSSHOptions {
        host: node_en.host,
        port: node_en.port as u16,
        username: node_en.account,
        allow_insecure_algos: Some(true),
        auth: genesis_common::SSHTargetAuth::Password(SshTargetPasswordAuth {
            password: node_en.password,
        }),
        pty_request: PtyRequest {
            term: query.term.clone(),
            width: query.w,
            height: query.h,
        },
    };
    // step2. connect
    let mut ssh_manager = SSHProcessManager::new(uuid).with_recorder_param(
        &SHARED_APP_CONFIG.read().await.server.recording_path,
        &query.term,
        query.h,
        query.w,
    )?;
    let abort_sc = ssh_manager.get_abort_sc();
    let abort_rc = ssh_manager.get_abort_rc();
    let (server_sender, xs, see) = ssh_manager.run(option).await?;
    let res = ws.on_upgrade(move |socket| {
        let session_id = uuid;
        async move {
            let (sender, receiver) = socket.split();
            let s_c = SSHSessionCtx::new(session_id).with_on_close(Some(Box::new(move || {
                let _ = abort_sc.send(true);
                debug!(session_id=%session_id,"send close session channel")
            })));
            let _ = GLOBAL_MANAGER
                .session_manager
                .register(session_id, Arc::new(Mutex::new(s_c)))
                .await;
            let _ = tokio::join!(
                write_to_client(session_id, sender, xs),
                read_to_server(abort_rc, session_id, receiver, server_sender, see),
            );
            let _ = GLOBAL_MANAGER.session_manager.remove(session_id).await;
        }
    });
    Ok(res)
}

async fn read_to_server(
    mut abort_rc: watch::Receiver<bool>,
    uuid: Uuid,
    receiver: SplitStream<WebSocket>,
    sender: UnboundedSender<Bytes>,
    see: UnboundedSender<ServerExtraEnum>,
) {
    debug!(session_id=%uuid, "start ws receiver");
    let mut receiver = receiver.fuse();
    loop {
        tokio::select! {
            ws_msg = receiver.next() => match ws_msg {
                Some(Ok(message)) => {
                    match message {
                        Message::Text(text) => match serde_json::from_str::<Envelope>(&text) {
                            Ok(env) =>match env.r#type {
                                EnvelopeType::Raw => {
                                    if let Err(err) = sender.send(Bytes::from(env.payload)) {
                                    debug!(session_id=%uuid,"convert input to envelope error:{}",err);
                                    break;
                                }
                                }
                                EnvelopeType::WindowSize => {
                                    let sp = env.payload.split(":").filter_map(|e| e.parse::<u32>().ok()).collect::<Vec<u32>>();
                                    if sp.len() != 2 {
                                        debug!(session_id=%uuid,"received windowSize message format error:{}",env.payload);
                                        continue;
                                    }
                                    let _ = see.send(
                                        ServerExtraEnum::ChannelOperation(ChannelOperation::ResizePty(
                                                genesis_ssh::PtyRequest{
                                                 term: "xterm".to_string(),
                                                col_width: sp[0],
                                                row_height: sp[1],
                                                pix_width: 0,
                                                pix_height: 0,
                                                modes: vec![],
                                            })));
                                    debug!(session_id=%uuid,"received WindowSize message:{}",env.payload);
                                }
                            }
                            Err(err) => {
                                debug!(session_id=%uuid,"serde deserialize error:{}",err);
                                break;
                            }
                        },
                        Message::Binary(_) => {
                            debug!(session_id=%uuid,"binary type is not supported");
                            break;
                        }
                        Message::Close(_) => {
                            debug!(session_id=%uuid,"client close ws");
                            break;
                        }
                        //ping or pong not process
                        _ => {}
                    }
                },
                Some(Err(e)) => {
                    debug!(session_id=%uuid,"error reading websocket: {e}");
                    break;
                },
                None => {
                    debug!(session_id=%uuid,"websocket closed");
                    break;
                },
            },
            _ = abort_rc.changed() => {
                if *abort_rc.borrow() {
                    debug!(session_id=%uuid,"received abort");
                    break;
                }
            }
        }
    }
    info!(session_id=%uuid,"end ws receiver");
}

async fn write_to_client(
    uuid: Uuid,
    mut sender: SplitSink<WebSocket, Message>,
    mut rec: broadcast::Receiver<ExecuteState>,
) {
    loop {
        tokio::select! {
            data = rec.recv() => match data{
                Ok(state) =>  match state {
                    ExecuteState::ExecutedBytes(bytes) => {
                        match str::from_utf8(&bytes) {
                            Ok(data) => {
                                let x = Envelope {
                                    version: "1.0".into(),
                                    r#type: EnvelopeType::Raw,
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
                    // TODO match other
                    _ => {
                    },
                },
                Err(e) => {
                    info!(session_id=%uuid,"receive error: {}",e);
                    break
                },
            }
        }
    }
    info!(session_id=%uuid,"end ws writer");
}
