use axum::{extract::ws::WebSocketUpgrade, response::Response};

use crate::error::AppError;
use crate::service::guacamole::process_double_axum;
use genesis_process::guacamole::constants::*;
use genesis_process::guacamole::process;
use tracing::error;

pub async fn handler_guacamole(
    ws: WebSocketUpgrade,
    // Query(bq): Query<SSHConnParams>,
    // State(state): State<AppState>,
) -> Result<Response, AppError> {
    let mut config = process::Configuration::new("rdp")
        .with(GUA_HOSTNAME, "10.0.2.4")
        .with(GUA_HOST_PORT, "3389")
        .with(GUA_USERNAME, "administrator")
        .with(GUA_PASSWORD, "Tcdn@2007")
        .with(GUA_WIDTH, "1024")
        .with(GUA_HEIGHT, "768")
        .with(GUA_DPI, "96");
    for (key, value) in default_rdp_properties() {
        config = config.with(key, value);
    }
    let uuid = uuid::Uuid::new_v4();
    let tunnel = process::Tunnel::connect("127.0.0.1:14822", config)
        .await
        .map_err(|e| {
            error!("{}", e);
            e.to_string()
        })?;
    let res = ws
        .protocols(["guacamole"])
        .on_upgrade(move |socket| async move {
            process_double_axum(
                uuid,
                socket,
                tunnel,
                Some(|e| {
                    error!("on error: {}", e);
                }),
            )
            .await;
        });
    Ok(res)
}
