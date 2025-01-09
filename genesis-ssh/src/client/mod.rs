mod channel_direct_tcpip;
mod channel_session;
mod error;
mod handler;
use std::borrow::Cow;
use std::collections::HashMap;
use std::io;
use std::net::ToSocketAddrs;
use std::sync::Arc;

use anyhow::Result;
use bytes::Bytes;
use channel_direct_tcpip::DirectTCPIPChannel;
use channel_session::SessionChannel;
pub use error::SshClientError;
use futures::pin_mut;
use genesis_common::{EventHub, NotifyEnum};
use genesis_common::{SSHTargetAuth, SessionId, TargetSSHOptions};
use handler::ClientHandler;
use russh::client::Handle;
use russh::keys::key::PublicKey;
use russh::{kex, Preferred, Sig};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::{oneshot, watch, Mutex};
use tokio::task::JoinHandle;
use tracing::*;
use uuid::Uuid;

use self::handler::ClientHandlerEvent;
use super::{ChannelOperation, DirectTCPIPParams};
use crate::client::handler::ClientHandlerError;
use crate::ForwardedTcpIpParams;

#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
    #[error("Host key mismatch")]
    HostKeyMismatch {
        received_key_type: String,
        received_key_base64: String,
        known_key_type: String,
        known_key_base64: String,
    },
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Key(#[from] russh::keys::Error),
    #[error(transparent)]
    Ssh(#[from] russh::Error),
    #[error("Could not resolve address")]
    Resolve,
    #[error("Internal error")]
    Internal,
    #[error("Aborted")]
    Aborted,
    #[error("Authentication failed")]
    Authentication,
}

#[derive(Debug)]
pub enum RCEvent {
    State(RCState),
    Output(Uuid, Bytes),
    Success(Uuid),
    ChannelFailure(Uuid),
    Eof(Uuid),
    Close(Uuid),
    Error(anyhow::Error),
    ExitStatus(Uuid, u32),
    ExitSignal {
        channel: Uuid,
        signal_name: Sig,
        core_dumped: bool,
        error_message: String,
        lang_tag: String,
    },
    ExtendedData {
        channel: Uuid,
        data: Bytes,
        ext: u32,
    },
    ConnectionError(ConnectionError),
    // ForwardedTCPIP(Uuid, DirectTCPIPParams),
    Done,
    HostKeyReceived(PublicKey),
    HostKeyUnknown(PublicKey, oneshot::Sender<bool>),
    ForwardedTcpIp(Uuid, ForwardedTcpIpParams),
    X11(Uuid, String, u32),
}

pub type RCCommandReply = oneshot::Sender<Result<(), SshClientError>>;

#[derive(Clone, Debug)]
pub enum RCCommand {
    Connect(TargetSSHOptions),
    Channel(Uuid, ChannelOperation),
    ForwardTCPIP(String, u32),
    CancelTCPIPForward(String, u32),
    Disconnect,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RCState {
    NotInitialized,
    Connecting,
    Connected,
    Disconnected,
}

#[derive(Debug)]
enum InnerEvent {
    RCCommand(RCCommand, Option<RCCommandReply>),
    ClientHandlerEvent(ClientHandlerEvent),
}

pub struct RemoteClient {
    id: SessionId,
    tx: UnboundedSender<RCEvent>,
    session: Option<Arc<Mutex<Handle<ClientHandler>>>>,
    channel_pipes: Arc<Mutex<HashMap<Uuid, UnboundedSender<ChannelOperation>>>>,
    pending_ops: Vec<(Uuid, ChannelOperation)>,
    pending_forwards: Vec<(String, u32)>,
    state: RCState,
    abort_rx: UnboundedReceiver<()>,
    inner_event_rx: UnboundedReceiver<InnerEvent>,
    inner_event_tx: UnboundedSender<InnerEvent>,
    child_tasks: Vec<JoinHandle<Result<(), SshClientError>>>,
}

pub struct RemoteClientHandles {
    pub event_rx: UnboundedReceiver<RCEvent>,
    pub command_tx: UnboundedSender<(RCCommand, Option<RCCommandReply>)>,
    pub abort_tx: UnboundedSender<()>,
}

impl RemoteClient {
    pub fn create(id: SessionId) -> io::Result<RemoteClientHandles> {
        let (event_tx, event_rx) = unbounded_channel();
        let (command_tx, mut command_rx) = unbounded_channel();
        let (abort_tx, abort_rx) = unbounded_channel();
        let (inner_event_tx, inner_event_rx) = unbounded_channel();

        let this = Self {
            id,
            tx: event_tx,
            session: None,
            channel_pipes: Arc::new(Mutex::new(HashMap::new())),
            pending_ops: vec![],
            pending_forwards: vec![],
            state: RCState::NotInitialized,
            inner_event_rx,
            inner_event_tx: inner_event_tx.clone(),
            child_tasks: vec![],
            abort_rx,
        };

        tokio::spawn(
            {
                async move {
                    // 接受
                    while let Some((e, response)) = command_rx.recv().await {
                        inner_event_tx.send(InnerEvent::RCCommand(e, response))?
                    }
                    Ok::<(), anyhow::Error>(())
                }
            }
            .instrument(Span::current()),
        );

        this.start()?;

        Ok(RemoteClientHandles {
            event_rx,
            command_tx,
            abort_tx,
        })
    }

    fn set_disconnected(&mut self) {
        self.session = None;
        for (id, op) in self.pending_ops.drain(..) {
            if let ChannelOperation::OpenShell = op {
                let _ = self.tx.send(RCEvent::Close(id));
            }
            if let ChannelOperation::OpenDirectTCPIP { .. } = op {
                let _ = self.tx.send(RCEvent::Close(id));
            }
        }
        let _ = self.set_state(RCState::Disconnected);
        let _ = self.tx.send(RCEvent::Done);
    }

    fn set_state(&mut self, state: RCState) -> Result<(), SshClientError> {
        self.state = state.clone();
        self.tx
            .send(RCEvent::State(state))
            .map_err(|_| SshClientError::MpscError)?;
        Ok(())
    }

    async fn apply_channel_op(
        &mut self,
        channel_id: Uuid,
        op: ChannelOperation,
    ) -> Result<(), SshClientError> {
        if self.state != RCState::Connected {
            self.pending_ops.push((channel_id, op));
            return Ok(());
        }

        match op {
            ChannelOperation::OpenShell => {
                self.open_shell(channel_id).await?;
            }
            ChannelOperation::OpenDirectTCPIP(params) => {
                self.open_direct_tcpip(channel_id, params).await?;
            }
            op => {
                let mut channel_pipes = self.channel_pipes.lock().await;
                match channel_pipes.get(&channel_id) {
                    Some(tx) => {
                        if tx.send(op).is_err() {
                            channel_pipes.remove(&channel_id);
                        }
                    }
                    None => debug!(channel=%channel_id, "operation for unknown channel"),
                }
            }
        }
        Ok(())
    }

    pub fn start(mut self) -> io::Result<JoinHandle<anyhow::Result<()>>> {
        let name = format!("SSH {} client commands", self.id);
        tokio::task::Builder::new().name(&name).spawn(
            async move {
                async {
                    loop {
                        tokio::select! {
                            Some(event) = self.inner_event_rx.recv() => {
                                debug!(event=?event, "event");
                                if self.handle_event(event).await? {
                                    break
                                }
                            }
                            Some(_) = self.abort_rx.recv() => {
                                debug!("Abort requested");
                                self.disconnect().await;
                                break
                            }
                        };
                    }
                    Ok::<(), anyhow::Error>(())
                }
                .await
                .map_err(|error| {
                    error!(?error, "error in command loop");
                    let err = anyhow::anyhow!("Error in command loop: {error}");
                    let _ = self.tx.send(RCEvent::Error(error));
                    err
                })?;
                info!("Client session closed");
                Ok::<(), anyhow::Error>(())
            }
            .instrument(Span::current()),
        )
    }

    async fn handle_event(&mut self, event: InnerEvent) -> Result<bool> {
        match event {
            InnerEvent::RCCommand(cmd, reply) => {
                let result = self.handle_command(cmd).await;
                let brk = matches!(result, Ok(true));
                if let Some(reply) = reply {
                    let _ = reply.send(result.map(|_| ()));
                }
                return Ok(brk);
            }
            InnerEvent::ClientHandlerEvent(client_event) => {
                debug!("Client handler event: {:?}", client_event);
                match client_event {
                    ClientHandlerEvent::Disconnect => {
                        self._on_disconnect().await?;
                    }
                    ClientHandlerEvent::ForwardedTcpIp(channel, params) => {
                        info!("New forwarded connection: {params:?}");
                        let id = self.setup_server_initiated_channel(channel).await?;
                        let _ = self.tx.send(RCEvent::ForwardedTcpIp(id, params));
                    }
                    ClientHandlerEvent::X11(channel, originator_address, originator_port) => {
                        info!("New X11 connection from {originator_address}:{originator_port:?}");
                        let id = self.setup_server_initiated_channel(channel).await?;
                        let _ = self
                            .tx
                            .send(RCEvent::X11(id, originator_address, originator_port));
                    }
                    event => {
                        error!(?event, "Unhandled client handler event");
                    }
                }
            }
        }
        Ok(false)
    }

    async fn setup_server_initiated_channel(
        &mut self,
        channel: russh::Channel<russh::client::Msg>,
    ) -> Result<Uuid> {
        let id = Uuid::new_v4();

        let (tx, rx) = unbounded_channel();
        self.channel_pipes.lock().await.insert(id, tx);

        let session_channel = SessionChannel::new(channel, id, rx, self.tx.clone(), self.id);

        self.child_tasks.push(
            tokio::task::Builder::new()
                .name(&format!("SSH {} {:?} ops", self.id, id))
                .spawn(session_channel.run())?,
        );

        Ok(id)
    }

    async fn handle_command(&mut self, cmd: RCCommand) -> Result<bool, SshClientError> {
        match cmd {
            RCCommand::Connect(options) => match self.connect(options).await {
                Ok(_) => {
                    self.set_state(RCState::Connected)
                        .map_err(SshClientError::other)?;
                    let ops = self.pending_ops.drain(..).collect::<Vec<_>>();
                    for (id, op) in ops {
                        self.apply_channel_op(id, op).await?;
                    }
                    let forwards = self.pending_forwards.drain(..).collect::<Vec<_>>();
                    for (address, port) in forwards {
                        self.tcpip_forward(address, port).await?;
                    }
                }
                Err(e) => {
                    debug!("Connect error: {}", e);
                    let _ = self.tx.send(RCEvent::ConnectionError(e));
                    self.set_disconnected();
                    return Ok(true);
                }
            },
            RCCommand::Channel(ch, op) => {
                self.apply_channel_op(ch, op).await?;
            }
            RCCommand::ForwardTCPIP(address, port) => {
                self.tcpip_forward(address, port).await?;
            }
            RCCommand::CancelTCPIPForward(address, port) => {
                self.cancel_tcpip_forward(address, port).await?;
            }
            RCCommand::Disconnect => {
                self.disconnect().await;
                return Ok(true);
            }
        }
        Ok(false)
    }

    async fn connect(&mut self, ssh_options: TargetSSHOptions) -> Result<(), ConnectionError> {
        let address_str = format!("{}:{}", ssh_options.host, ssh_options.port);
        let address = match address_str
            .to_socket_addrs()
            .map_err(ConnectionError::Io)
            .and_then(|mut x| x.next().ok_or(ConnectionError::Resolve))
        {
            Ok(address) => address,
            Err(error) => {
                error!(?error, address=%address_str, "Cannot resolve target address");
                self.set_disconnected();
                return Err(error);
            }
        };

        info!(?address, username = &ssh_options.username[..], "Connecting");
        let algos = if ssh_options.allow_insecure_algos.unwrap_or(false) {
            Preferred {
                kex: Cow::Borrowed(&[
                    kex::CURVE25519,
                    kex::CURVE25519_PRE_RFC_8731,
                    kex::ECDH_SHA2_NISTP256,
                    kex::ECDH_SHA2_NISTP384,
                    kex::ECDH_SHA2_NISTP521,
                    kex::DH_G16_SHA512,
                    kex::DH_G14_SHA256, // non-default
                    kex::DH_G14_SHA256,
                    kex::DH_G1_SHA1, // non-default
                    kex::EXTENSION_SUPPORT_AS_CLIENT,
                    kex::EXTENSION_SUPPORT_AS_SERVER,
                    kex::EXTENSION_OPENSSH_STRICT_KEX_AS_CLIENT,
                    kex::EXTENSION_OPENSSH_STRICT_KEX_AS_SERVER,
                ]),
                ..<_>::default()
            }
        } else {
            Preferred::default()
        };

        let config = russh::client::Config {
            preferred: algos,
            ..Default::default()
        };
        let config = Arc::new(config);

        let (event_tx, mut event_rx) = unbounded_channel();
        let handler = ClientHandler {
            ssh_options: ssh_options.clone(),
            event_tx,
            session_id: self.id,
        };

        let fut_connect = russh::client::connect(config, address, handler);
        pin_mut!(fut_connect);

        loop {
            tokio::select! {
                Some(event) = event_rx.recv() => {
                    match event {
                        ClientHandlerEvent::HostKeyReceived(key) => {
                            self.tx.send(RCEvent::HostKeyReceived(key)).map_err(|_| ConnectionError::Internal)?;
                        }
                        ClientHandlerEvent::HostKeyUnknown(key, reply) => {
                            self.tx.send(RCEvent::HostKeyUnknown(key, reply)).map_err(|_| ConnectionError::Internal)?;
                        }
                        _ => {}
                    }
                }
                Some(_) = self.abort_rx.recv() => {
                    info!("Abort requested");
                    self.set_disconnected();
                    return Err(ConnectionError::Aborted)
                }
                session = &mut fut_connect => {
                    let mut session = match session {
                        Ok(session) => session,
                        Err(error) => {
                            let connection_error = match error {
                                ClientHandlerError::ConnectionError(e) => e,
                                ClientHandlerError::Ssh(e) => ConnectionError::Ssh(e),
                                ClientHandlerError::Internal => ConnectionError::Internal,
                            };
                            error!(error=?connection_error, "Connection error");
                            return Err(connection_error);
                        }
                    };
                    let mut auth_result = false;
                    match ssh_options.auth {
                        SSHTargetAuth::Password(auth) => {
                            auth_result = session
                                .authenticate_password(ssh_options.username.clone(), auth.password)
                                .await?;
                            if auth_result {
                                debug!(username=&ssh_options.username[..], "Authenticated with password");
                            }
                        }
                        SSHTargetAuth::PublicKey(_) => {
                            error!(username=&ssh_options.username[..], auth_result, "Authenticated with publicKey");
                            return Err(ConnectionError::Authentication);
                        }
                    }

                    if !auth_result {
                        error!("Auth rejected");
                        let _ = session
                            .disconnect(russh::Disconnect::ByApplication, "", "")
                            .await;
                        return Err(ConnectionError::Authentication);
                    }

                    self.session = Some(Arc::new(Mutex::new(session)));

                    info!(?address, "Connected");

                    tokio::spawn({
                        let inner_event_tx = self.inner_event_tx.clone();
                        async move {
                            while let Some(e) = event_rx.recv().await {
                                info!("{:?}", e);
                                inner_event_tx.send(InnerEvent::ClientHandlerEvent(e))?
                            }
                            Ok::<(), anyhow::Error>(())
                        }
                    }.instrument(Span::current()));

                    return Ok(())
                }
            }
        }
    }

    async fn open_shell(&mut self, channel_id: Uuid) -> Result<(), SshClientError> {
        if let Some(session) = &self.session {
            let session = session.lock().await;
            let channel = session.channel_open_session().await?;

            let (tx, rx) = unbounded_channel();
            self.channel_pipes.lock().await.insert(channel_id, tx);

            let channel = SessionChannel::new(channel, channel_id, rx, self.tx.clone(), self.id);
            self.child_tasks.push(
                tokio::task::Builder::new()
                    .name(&format!("SSH {} {:?} ops", self.id, channel_id))
                    .spawn(channel.run())
                    .map_err(|e| SshClientError::Other(Box::new(e)))?,
            );
        }
        Ok(())
    }

    async fn open_direct_tcpip(
        &mut self,
        channel_id: Uuid,
        params: DirectTCPIPParams,
    ) -> Result<(), SshClientError> {
        if let Some(session) = &self.session {
            let session = session.lock().await;
            let channel = session
                .channel_open_direct_tcpip(
                    params.host_to_connect,
                    params.port_to_connect,
                    params.originator_address,
                    params.originator_port,
                )
                .await?;

            let (tx, rx) = unbounded_channel();
            self.channel_pipes.lock().await.insert(channel_id, tx);

            let channel =
                DirectTCPIPChannel::new(channel, channel_id, rx, self.tx.clone(), self.id);
            self.child_tasks.push(
                tokio::task::Builder::new()
                    .name(&format!("SSH {} {:?} ops", self.id, channel_id))
                    .spawn(channel.run())
                    .map_err(|e| SshClientError::Other(Box::new(e)))?,
            );
        }
        Ok(())
    }

    async fn tcpip_forward(&mut self, address: String, port: u32) -> Result<(), SshClientError> {
        if let Some(session) = &self.session {
            let mut session = session.lock().await;
            session.tcpip_forward(address, port).await?;
        } else {
            self.pending_forwards.push((address, port));
        }
        Ok(())
    }

    async fn cancel_tcpip_forward(
        &mut self,
        address: String,
        port: u32,
    ) -> Result<(), SshClientError> {
        if let Some(session) = &self.session {
            let session = session.lock().await;
            session.cancel_tcpip_forward(address, port).await?;
        } else {
            self.pending_forwards
                .retain(|x| x.0 != address || x.1 != port);
        }
        Ok(())
    }

    async fn disconnect(&mut self) {
        if let Some(session) = &mut self.session {
            let _ = session
                .lock()
                .await
                .disconnect(russh::Disconnect::ByApplication, "", "")
                .await;
            self.set_disconnected();
        }
    }

    async fn _on_disconnect(&mut self) -> Result<()> {
        self.set_disconnected();
        Ok(())
    }
}

impl Drop for RemoteClient {
    fn drop(&mut self) {
        for task in self.child_tasks.drain(..) {
            task.abort();
        }
        info!("Closed connection");
        debug!("Dropped");
    }
}

pub async fn start_ssh_connect(
    uuid: Uuid,
    option: TargetSSHOptions,
) -> Result<(
    EventHub<Bytes>,
    UnboundedSender<Bytes>,
    watch::Receiver<NotifyEnum>,
)> {
    // step1. start connect
    let mut handle = RemoteClient::create(uuid)?;
    let (tx, rx) = oneshot::channel();
    handle
        .command_tx
        .send((RCCommand::Connect(option), Some(tx)))?;
    let _ = rx.await?;
    // step2. start open channel
    let channel_id = Uuid::new_v4();
    let message = (
        RCCommand::Channel(channel_id, ChannelOperation::OpenShell),
        None,
    );
    info!(channel_id=%channel_id,"open channel");
    handle.command_tx.send(message)?;
    // step3. request pty
    let message = (
        RCCommand::Channel(
            channel_id,
            ChannelOperation::RequestPty(crate::PtyRequest {
                term: "xterm".into(),
                col_width: 80,
                row_height: 24,
                pix_width: 0,
                pix_height: 0,
                modes: vec![],
            }),
        ),
        None,
    );
    info!(channel_id=%channel_id,"request pty");
    handle.command_tx.send(message)?;
    // step4. start shell
    let message = (
        RCCommand::Channel(channel_id, ChannelOperation::RequestShell),
        None,
    );
    info!(channel_id=%channel_id,"request shell");
    handle.command_tx.send(message)?;

    // step5 create event hub
    let (hub, sender) = EventHub::setup();
    let (notify_sender, notify_receiver) = watch::channel(NotifyEnum::INIT);
    // step6. start ssh channel
    tokio::spawn(async move {
        while let Some(e) = handle.event_rx.recv().await {
            match e {
                RCEvent::Output(_, bytes) => {
                    let _ = sender.send_once(bytes).await;
                }
                RCEvent::ConnectionError(e) => {
                    error!("connection error:{:?}", e);
                    let _ = notify_sender.send(NotifyEnum::ERROR(e.to_string()));
                }
                RCEvent::State(RCState::Connected) => {
                    info!("state Connected");
                    let _ = notify_sender.send(NotifyEnum::SUCCESS);
                }
                _ => {
                    info!("receive event : {:?}", e);
                }
            }
        }
    });
    let (tx, mut rx) = unbounded_channel();

    tokio::spawn(async move {
        while let Some(e) = rx.recv().await {
            handle
                .command_tx
                .send((
                    RCCommand::Channel(channel_id, ChannelOperation::Data(e)),
                    None,
                ))
                .unwrap();
        }
    });
    anyhow::Ok((hub, tx, notify_receiver.clone()))
}

#[cfg(test)]
mod tests {
    use super::RemoteClient;
    use super::*;
    use genesis_common::SshTargetPasswordAuth;
    use tokio::io::AsyncWriteExt;
    use uuid::Uuid;
    #[tokio::test]
    #[ignore]
    async fn test_remote_client() {
        tracing_subscriber::fmt().init();
        let uuid = Uuid::new_v4();
        info!("test remote client uuid:{}", uuid.to_string());
        let rc = RemoteClient::create(uuid);
        match rc {
            Ok(mut handle) => {
                // step1. create connect
                let (tx, rx) = oneshot::channel();
                info!("start connect");
                let message = (
                    RCCommand::Connect(TargetSSHOptions {
                        host: "10.0.1.52".into(),
                        port: 22,
                        username: "root".into(),
                        allow_insecure_algos: Some(true),
                        auth: SSHTargetAuth::Password(SshTargetPasswordAuth {
                            password: "1qaz2wsx".into(),
                        }),
                    }),
                    Some(tx),
                );
                handle.command_tx.send(message).unwrap();
                info!("wait connct command");
                match rx.await {
                    Ok(r) => match r {
                        Ok(_) => {}
                        Err(e) => {
                            error!("send command error: {}", e);
                            return;
                        }
                    },
                    Err(e) => {
                        error!("await send command error:{}", e);
                        return;
                    }
                };
                info!("connect success");

                // step2. start shell
                let channel_id = Uuid::new_v4();
                let message = (
                    RCCommand::Channel(channel_id, ChannelOperation::OpenShell),
                    None,
                );
                info!(channel_id=%channel_id,"open channel");
                handle.command_tx.send(message).unwrap();

                // step3. request pty
                let message = (
                    RCCommand::Channel(
                        channel_id,
                        ChannelOperation::RequestPty(crate::PtyRequest {
                            term: "xterm".into(),
                            col_width: 100,
                            row_height: 10,
                            pix_width: 0,
                            pix_height: 0,
                            modes: vec![],
                        }),
                    ),
                    None,
                );
                info!(channel_id=%channel_id,"request pty");
                handle.command_tx.send(message).unwrap();
                // step4. start shell
                let message = (
                    RCCommand::Channel(channel_id, ChannelOperation::RequestShell),
                    None,
                );
                info!(channel_id=%channel_id,"request shell");
                handle.command_tx.send(message).unwrap();

                let handle = tokio::spawn(async move {
                    // io_std feature
                    let mut out = tokio::io::stdout();
                    while let Some(e) = handle.event_rx.recv().await {
                        match e {
                            crate::RCEvent::Output(_, bytes) => {
                                let _ = out.write_all(&bytes).await;
                                let _ = out.flush().await;
                                let character = '#';
                                let byte: u8 = character as u8;
                                if bytes.contains(&byte) {
                                    return;
                                }
                            }
                            _ => {
                                info!("receive envent : {:?}", e);
                            }
                        }
                    }
                });
                let _ = tokio::join!(handle);
                info!("success test");
            }
            Err(e) => error!("error create remote client:{:?}", e),
        }
    }
}
