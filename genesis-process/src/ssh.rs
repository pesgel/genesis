use std::{io::Write, sync::Arc, time::Duration};

use bytes::Bytes;
use genesis_common::{EventSubscription, TargetSSHOptions};
use genesis_ssh::start_ssh_connect_with_state;
use tokio::{
    select,
    sync::{
        broadcast,
        mpsc::{unbounded_channel, UnboundedSender},
        watch, Mutex,
    },
};
use tracing::{debug, error};
use uuid::Uuid;

use crate::{recording::Recorder, ExecuteState, Pipe, PipeManger};

pub struct SSHProcessManager {
    uniq_id: Uuid,
    ssh_cmd_wait_times: u8,
    abort_sc: watch::Sender<bool>,
    abort_rc: watch::Receiver<bool>,
    recorder: Arc<Mutex<Option<Recorder>>>,
}

impl SSHProcessManager {
    pub fn new(uniq_id: Uuid) -> Self {
        let (abort_sc, abort_rc) = watch::channel(false);
        Self {
            uniq_id,
            abort_sc,
            abort_rc,
            ssh_cmd_wait_times: 100,
            recorder: Arc::new(Mutex::new(None)),
        }
    }
    pub fn with_recorder_param(
        mut self,
        save_path: &str,
        term: &str,
        height: u8,
        width: u8,
    ) -> anyhow::Result<Self> {
        self.recorder = Arc::new(Mutex::new(Some(Recorder::new(
            &self.uniq_id.to_string(),
            save_path,
            term,
            height,
            width,
        )?)));
        anyhow::Ok(self)
    }

    /// set the maximum time to wait for ssh data to return
    pub fn with_ssh_cmd_wait_times(&mut self, times: u8) -> &mut Self {
        self.ssh_cmd_wait_times = times;
        self
    }
    /// get global abort channel
    pub fn get_abort_sc(&self) -> watch::Sender<bool> {
        self.abort_sc.clone()
    }

    pub fn get_abort_rc(&self) -> watch::Receiver<bool> {
        self.abort_rc.clone()
    }

    /// stop process
    pub fn stop_process(&self) {
        match self.abort_sc.send(true) {
            Ok(_) => {}
            Err(e) => {
                error!(session_id=%self.uniq_id,"execute:{} abort signal error:{}", self.uniq_id, e);
            }
        }
    }
}
impl SSHProcessManager {
    /// operation recording
    async fn do_recording(&self, recv: EventSubscription<Bytes>) {
        let uniq_id = self.uniq_id;
        let abort_sc = self.get_abort_sc();
        let mut abort_tx: watch::Receiver<bool> = self.abort_rc.clone();
        let mut receiver = recv.unbox();
        if let Some(mut recorder) = self.recorder.lock().await.take() {
            tokio::spawn(async move {
                loop {
                    select! {
                        flag = abort_tx.changed() => match flag {
                                Ok(_) => {
                                    if *abort_tx.borrow() {
                                        debug!(session_id=%uniq_id,"do_recording receive abort signal");
                                        break ;
                                    }
                                },
                                Err(e) => {
                                    error!(session_id=%uniq_id,"do_recording receive abort signal error: {:?}",e);
                                    break ;
                                },
                            },
                        _ = tokio::time::sleep(Duration::from_secs(3)) => {
                            match recorder.flush() {
                                    Ok(_) => {},
                                    Err(e) => {
                                        error!(session_id=%uniq_id,"do_recording flush error: {:?}",e);
                                        break;
                                    }
                            }
                        }
                        rb = receiver.recv() => match rb {
                            None  => {
                                match abort_sc.send(true) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        error!(session_id=%uniq_id,"execute:{} abort signal error:{}", uniq_id, e);
                                    }
                                }
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
    ) -> anyhow::Result<(UnboundedSender<Bytes>, broadcast::Receiver<ExecuteState>)> {
        let (hub, sender) =
            start_ssh_connect_with_state(self.uniq_id, ssh_option, Some(self.get_abort_sc()))
                .await?;
        // step2. Two-way binary stream copy
        let receiver = hub.subscribe(|_| true).await;
        let (sc, in_rc) = unbounded_channel::<Bytes>();
        let (psc, _) = unbounded_channel::<Bytes>();
        let in_pipe = Pipe::new(psc, in_rc);
        let out_pipe = Pipe::new(sender, receiver.unbox());
        let manager = PipeManger::new(self.ssh_cmd_wait_times, self.uniq_id.to_string());
        // step3.start interactive
        let (broadcast_sender, broadcast_receiver) = broadcast::channel::<ExecuteState>(2048);
        let new_manager = Arc::new(manager);
        let _ = new_manager
            .clone()
            .do_interactive(
                in_pipe,
                out_pipe,
                broadcast_sender,
                self.abort_rc.clone(),
                self.get_abort_sc(),
            )
            .await;
        // step5. cmd & recording process
        self.do_recording(hub.subscribe(|_| true).await).await;
        anyhow::Ok((sc, broadcast_receiver))
    }
}
