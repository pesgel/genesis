//! 提供SSH相关聚合操作
use bytes::Bytes;
use derive_builder::Builder;
use genesis_common::{EventSubscription, TargetSSHOptions};
use genesis_ssh::{start_ssh_connect_with_state, ServerExtraEnum};
use std::{io::Write, sync::Arc, time::Duration};
use tokio::{
    select,
    sync::{broadcast, mpsc::UnboundedSender, Mutex},
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};
use uuid::Uuid;

use crate::recording::Recorder;
use crate::sshm::pipe::{ExecuteState, PipeMangerBuilder};

#[derive(Default, Builder)]
#[builder(build_fn(private, name = "private_build"))]
#[builder(setter(into))]
/// 处理器
pub struct ChannelManager {
    uniq_id: Uuid,
    ps1_char: Vec<char>,
    wait_times: u8,
    ctx: CancellationToken,
    #[builder(setter(skip))]
    recorder: Arc<Mutex<Option<Recorder>>>,
}

impl ChannelManagerBuilder {
    pub fn build(&mut self) -> anyhow::Result<ChannelManager> {
        if self.ps1_char.is_none() {
            self.ps1_char = Some(vec!['#', '$', '>'])
        }
        Ok(self.private_build()?)
    }
}

impl ChannelManager {
    pub fn get_ctx(&self) -> CancellationToken {
        self.ctx.clone()
    }

    pub fn stop_process(&self) {
        self.ctx.cancel();
    }

    pub fn with_recorder(&mut self, r: Recorder) -> &mut Self {
        self.recorder = Arc::new(Mutex::new(Some(r)));
        self
    }
}
impl ChannelManager {
    /// 记录日志数据
    async fn do_recording(&self, recv: EventSubscription<Bytes>) {
        let mut recorder = match self.recorder.lock().await.take() {
            Some(r) => r,
            None => return,
        };
        let uniq_id = self.uniq_id;
        let clone_ctx = self.ctx.clone();
        let mut receiver = recv.unbox();
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
        debug!(session_id=%self.uniq_id,"do_recording end");
    }

    /// run ssh process
    pub async fn run(
        &mut self,
        ssh_option: TargetSSHOptions,
    ) -> anyhow::Result<(
        UnboundedSender<Bytes>,
        broadcast::Receiver<ExecuteState>,
        UnboundedSender<ServerExtraEnum>,
    )> {
        // step1. 建立ssh连接
        let (hub, sender, see) =
            start_ssh_connect_with_state(self.uniq_id, ssh_option, Some(self.ctx.clone())).await?;
        // step2. 创建双向拷贝通道
        let receiver = hub.subscribe(|_| true).await;
        let pipe_manager = Arc::new(
            PipeMangerBuilder::default()
                .uniq_id(self.uniq_id)
                .wait_times(self.wait_times)
                .ctx(self.ctx.clone())
                .ps1_char(self.ps1_char.clone())
                .build()?,
        );
        // step3. 双向处理数据
        let (sc, br) = pipe_manager.do_interactive(sender, receiver.unbox())?;
        // step3. cmd & recording process
        self.do_recording(hub.subscribe(|_| true).await).await;
        anyhow::Ok((sc, br, see))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use genesis_common::{
        PtyRequest, SSHTargetAuth, SshTargetPasswordAuth, TargetSSHOptionsBuilder,
    };
    use tokio::time;
    #[tokio::test]
    async fn test_ssh_channel_run() -> anyhow::Result<()> {
        // step1. build channel
        let ctx = CancellationToken::new();
        let mut cm = ChannelManagerBuilder::default()
            .ctx(ctx.clone())
            .uniq_id(Uuid::new_v4())
            .wait_times(5)
            .build()?;
        // step2. ssh options
        let ssh_options = TargetSSHOptionsBuilder::default()
            .host("10.0.1.52")
            .port(22u16)
            .username("root")
            .auth(SSHTargetAuth::Password(SshTargetPasswordAuth {
                password: "1qaz2wsx".to_string(),
            }))
            .pty_request(PtyRequest::default())
            .build()
            .expect("unable to build channel manager");
        let (sender, mut receiver, _) = cm.run(ssh_options).await?;
        let sender_ctx = ctx.clone();
        tokio::spawn(async move {
            let mut x = 5;
            while x > 1 {
                x -= 1;
                tokio::time::sleep(Duration::from_secs(x)).await;
                match sender.send(Bytes::from(format!("pwd{x}\r"))) {
                    Ok(_) => continue,
                    Err(_) => break,
                };
            }
            sender_ctx.cancel();
        });
        let receiver_ctx = ctx.clone();
        tokio::spawn(async move {
            loop {
                select! {
                    _ = receiver_ctx.cancelled() => {
                        println!("receiver receive ctx cancelled");
                        break;
                    },
                    data = receiver.recv() => match data{
                        Ok(state) =>match state {
                            ExecuteState::ExecutedBytes(by) => println!("receive bytes: {:?}",String::from_utf8(by.to_vec())?),
                            _ => {
                                println!("receive other. terminated");
                                break;
                            }
                        }
                        Err(e) => {
                            println!("ctx recv error: {:?}",e);
                            break;
                        }
                    }
                }
            }
            anyhow::Ok(())
        });
        select! {
            _ = ctx.cancelled() => {
                println!("ctx cancelled");
            }
            // modify time to cancel
            _ = tokio::time::sleep(Duration::from_secs(3)) => {
                println!("timeout");
                ctx.cancel();
                time::sleep(Duration::from_secs(1)).await;
            }
        }
        println!("all end");
        Ok(())
    }
}
