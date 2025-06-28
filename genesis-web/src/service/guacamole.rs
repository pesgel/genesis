//! guacamole connection

use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use genesis_process::guacamole::process::Tunnel;
use std::sync::Arc;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    select,
    sync::Notify,
};
use tracing::{debug, error, info};
use uuid::Uuid;

const RATE44100: &[u8] = b"rate=44100,channels=2;";
const RATE22050: &[u8] = b"rate=22050,channels=2;";
const AUDIO_CODE: &[u8] = b"5.audio,1.1,31.audio/L16;";
const START_WITH: &[u8] = b"5.error";

/// 将 axum 的 WebSocket 与 guacamole 的 TCP 连接进行双向拷贝
pub async fn process_double_axum(
    uuid: Uuid,
    ws: WebSocket,
    tunnel: Tunnel,
    on_error: Option<impl Fn(String) + Send + Sync + 'static>,
) {
    let (reader_half, mut writer_half) = tokio::io::split(tunnel.reader.into_inner());

    let shutdown = Arc::new(Notify::new());
    let shutdown_clone = shutdown.clone();

    let (mut ws_sender, mut ws_receiver) = ws.split();
    // websocket -> guacamole
    let ws_to_tunnel = tokio::spawn(async move {
        loop {
            select! {
                en = ws_receiver.next() => match en {
                    Some(data) => match data {
                        Ok(msg) => {
                            match msg {
                                Message::Text(txt) => {
                                    if let Err(e) = writer_half.write_all(txt.as_bytes()).await {
                                        error!(session_id=%uuid,"write to guacamole err: {:?}", e);
                                        break;
                                    }
                                }
                                _ => {
                                    debug!(session_id=%uuid,"received unsupported ws message {:?}",msg);
                                }
                            }
                        }
                        Err(err) => {
                            error!(session_id=%uuid,"ws_receiver error: {:?}", err);
                            break;
                        }
                    },
                    None => {
                        break;
                    }
                }
            }
        }
        info!(session_id=%uuid,"ws to tunnel stop");
        shutdown_clone.notify_one();
    });

    // guacamole -> websocket
    let shutdown_clone = shutdown.clone();
    let tunnel_to_ws = tokio::spawn(async move {
        let mut reader = BufReader::new(reader_half);
        let mut buf = Vec::with_capacity(8192);
        let mut is_sent = false;
        loop {
            buf.clear();
            match reader.read_until(b';', &mut buf).await {
                Ok(0) => break,
                Ok(_) => {
                    if buf == RATE44100 || buf == RATE22050 {
                        continue;
                    }
                    if buf == AUDIO_CODE {
                        buf.extend_from_slice(RATE44100);
                    }
                    if !is_sent && buf.starts_with(START_WITH) {
                        if let Some(ref cb) = on_error {
                            cb(String::from_utf8_lossy(&buf).to_string());
                        }
                        is_sent = true;
                    }
                    if let Err(e) = ws_sender
                        .send(Message::Text(String::from_utf8_lossy(&buf).to_string()))
                        .await
                    {
                        debug!(session_id=%uuid,"guacamole send to websocket err: {:?}", e);
                        break;
                    }
                }
                Err(e) => {
                    error!(session_id=%uuid,"read tunnel data err: {:?}", e);
                    break;
                }
            }
        }
        shutdown_clone.notify_one();
    });
    shutdown.notified().await;
    let _ = ws_to_tunnel.await;
    let _ = tunnel_to_ws.await;
}
