//! 提供SSH相关聚合操作
use bytes::Bytes;
use genesis_common::{EventSubscription, TargetSSHOptions};
use genesis_ssh::{start_ssh_connect_with_state, ServerExtraEnum};
use std::{io::Write, sync::Arc, time::Duration};
use tokio::{
    select,
    sync::{
        broadcast,
        mpsc::{unbounded_channel, UnboundedSender},
        watch, Mutex,
    },
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};
use uuid::Uuid;

use crate::{recording::Recorder, ExecuteState, Pipe, PipeManger};

/// 处理器
pub struct ProcessManager {
    uniq_id: Uuid,
    wait_times: u8,
    ps1_char: Vec<char>,
    recorder: Arc<Mutex<Option<Recorder>>>,
    ctx: CancellationToken,
}

impl ProcessManager {
    pub fn new(uniq_id: Uuid) -> Self {
        let ctx = CancellationToken::new();
        Self::new_with_ctx(uniq_id, ctx)
    }

    pub fn new_with_ctx(uniq_id: Uuid, ctx: CancellationToken) -> Self {
        Self {
            uniq_id,
            ctx,
            wait_times: 50,
            recorder: Arc::new(Mutex::new(None)),
            ps1_char: vec!['#', '$', '>'],
        }
    }
    pub fn with_ps1_char(&mut self, chars: Vec<char>) -> &mut Self {
        self.ps1_char = chars;
        self
    }

    pub fn with_wait_times(&mut self, times: u8) -> &mut Self {
        self.wait_times = times;
        self
    }
    pub fn get_ctx(&self) -> CancellationToken {
        self.ctx.clone()
    }

    pub fn stop_process(&self) {
        self.ctx.cancel();
    }
}
impl ProcessManager {
    /// 记录日志数据
    async fn do_recording(&self, recv: EventSubscription<Bytes>) {
        let uniq_id = self.uniq_id;
        let clone_ctx = self.get_ctx();
        let mut receiver = recv.unbox();
        if let Some(mut recorder) = self.recorder.lock().await.take() {
            tokio::spawn(async move {
                loop {
                    select! {
                        _ = clone_ctx.cancelled() => {
                               debug!(session_id=%uniq_id,"do_recording receive abort signal");
                               break ;
                            },
                        _ = tokio::time::sleep(Duration::from_secs(3)) => {
                            match recorder.flush() {
                                    Ok(_) => {},
                                    Err(e) => {
                                        error!(session_id=%uniq_id,"do_recording flush error: {:?}",e);
                                        break;
                                    }
                            }
                        },
                        rb = receiver.recv() => match rb {
                            None  => {
                                break;
                            },
                            Some(bytes)=> {
                                match recorder.write_all(bytes.as_ref()) {
                                    Ok(_) => {},
                                    Err(e) => {
                                        error!(session_id=%uniq_id,"do_recording write error: {:?}",e);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                recorder.close();
            });
        }
        debug!(session_id=%self.uniq_id,"do_recording end");
    }

    /// start ssh process
    pub async fn run(
        &mut self,
        ssh_option: TargetSSHOptions,
    ) -> anyhow::Result<(
        UnboundedSender<Bytes>,
        broadcast::Receiver<ExecuteState>,
        UnboundedSender<ServerExtraEnum>,
    )> {
        let (hub, sender, see) =
            start_ssh_connect_with_state(self.uniq_id, ssh_option, Some(self.get_ctx())).await?;
        // step2. Two-way binary stream copy
        let receiver = hub.subscribe(|_| true).await;
        let (sc, in_rc) = unbounded_channel::<Bytes>();
        let (psc, _) = unbounded_channel::<Bytes>();
        let in_pipe = Pipe::new(psc, in_rc);
        let out_pipe = Pipe::new(sender, receiver.unbox());
        let mut manager = PipeManger::new(self.wait_times, self.uniq_id.to_string());
        manager.with_ps1_char(self.ps1_char.clone());
        // step3.start interactive
        let (broadcast_sender, broadcast_receiver) = broadcast::channel::<ExecuteState>(2048);
        let new_manager = Arc::new(manager);
        let _ = new_manager
            .clone()
            .do_interactive(in_pipe, out_pipe, broadcast_sender, self.get_ctx())
            .await;
        // step5. cmd & recording process
        self.do_recording(hub.subscribe(|_| true).await).await;
        anyhow::Ok((sc, broadcast_receiver, see))
    }
}
