use crate::adapter::cmd::guacamole::{GuacamoleConnParams, GuacamoleParams};
use crate::config::AppState;
use crate::error::AppError;
use crate::repo::sea::CredentialRepo;
use crate::service::guacamole::process_double_axum;
use axum::extract::{Query, State};
use axum::{extract::ws::WebSocketUpgrade, response::Response};
use genesis_process::guacamole::constants::*;
use genesis_process::guacamole::process;
use tracing::error;

pub async fn handler_guacamole(
    ws: WebSocketUpgrade,
    Query(gc): Query<GuacamoleConnParams>,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    // step1. 获取
    let params: GuacamoleParams = serde_json::from_str(&gc.params).map_err(|e| {
        println!("gua err: {e}");
        AppError::from(e)
    })?;
    let credential =
        CredentialRepo::get_credential_by_id(&state.conn, &params.permission_id).await?;

    let mut config = process::Configuration::new(&credential.protocol)
        .with(GUA_HOSTNAME, &credential.address)
        .with(GUA_HOST_PORT, &credential.port.to_string())
        .with(GUA_USERNAME, &credential.principal)
        .with(GUA_PASSWORD, &credential.credential)
        .with(GUA_WIDTH, &params.w.to_string())
        .with(GUA_HEIGHT, &params.h.to_string())
        .with(GUA_DPI, &params.dpi.to_string());
    for (key, value) in default_rdp_properties() {
        config = config.with(key, value);
    }
    let uuid = uuid::Uuid::new_v4();
    // TODO
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
