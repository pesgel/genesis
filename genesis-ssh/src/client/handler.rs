use genesis_common::{SessionId, TargetSSHOptions};
use russh::client::{Msg, Session};
use russh::keys::PublicKey;
use russh::Channel;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;
use tracing::*;

use crate::{ConnectionError, ForwardedTcpIpParams};

#[allow(dead_code)]
#[derive(Debug)]
pub enum ClientHandlerEvent {
    HostKeyReceived(PublicKey),
    HostKeyUnknown(PublicKey, oneshot::Sender<bool>),
    ForwardedTcpIp(Channel<Msg>, ForwardedTcpIpParams),
    X11(Channel<Msg>, String, u32),
    Disconnect,
}

#[allow(dead_code)]
pub struct ClientHandler {
    pub ssh_options: TargetSSHOptions,
    pub event_tx: UnboundedSender<ClientHandlerEvent>,
    pub session_id: SessionId,
}

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum ClientHandlerError {
    #[error("Connection error")]
    ConnectionError(ConnectionError),

    #[error("SSH")]
    Ssh(#[from] russh::Error),

    #[error("Internal error")]
    Internal,
}

impl russh::client::Handler for ClientHandler {
    type Error = ClientHandlerError;

    async fn check_server_key(&mut self, _: &PublicKey) -> Result<bool, Self::Error> {
        Ok(true)
    }

    async fn server_channel_open_forwarded_tcpip(
        &mut self,
        channel: Channel<Msg>,
        connected_address: &str,
        connected_port: u32,
        originator_address: &str,
        originator_port: u32,
        __session: &mut Session,
    ) -> Result<(), Self::Error> {
        let connected_address = connected_address.to_string();
        let originator_address = originator_address.to_string();
        let _ = self.event_tx.send(ClientHandlerEvent::ForwardedTcpIp(
            channel,
            ForwardedTcpIpParams {
                connected_address,
                connected_port,
                originator_address,
                originator_port,
            },
        ));
        Ok(())
    }

    async fn server_channel_open_x11(
        &mut self,
        channel: Channel<Msg>,
        originator_address: &str,
        originator_port: u32,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        let originator_address = originator_address.to_string();
        let _ = self.event_tx.send(ClientHandlerEvent::X11(
            channel,
            originator_address,
            originator_port,
        ));
        Ok(())
    }
}

impl Drop for ClientHandler {
    fn drop(&mut self) {
        let _ = self.event_tx.send(ClientHandlerEvent::Disconnect);
        debug!(session=%self.session_id, "Dropped");
    }
}
