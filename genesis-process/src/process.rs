//! process

use genesis_common::{SshTargetPasswordAuth, TargetSSHOptions};
use genesis_ssh::start_ssh_connect;
use tracing::info;
use uuid::Uuid;

#[allow(dead_code)]
pub async fn do_interactive() {
    info!("do_interactive");
    // step1. 建立到远程服务的连接并且初始化事件处理器
    let uuid = Uuid::new_v4();
    let option = TargetSSHOptions {
        host: "10.0.1.52".into(),
        port: 22,
        username: "root".into(),
        allow_insecure_algos: Some(true),
        auth: genesis_common::SSHTargetAuth::Password(SshTargetPasswordAuth {
            password: "1qaz2wsx".into(),
        }),
    };
    let (hub, _) = start_ssh_connect(uuid, option).await.unwrap();
    let _ = hub.subscribe(|_| true).await;

    let mut command_vec = Vec::new();
    let new_password = "%]m73MmQ";
    //let old_password = "#wR61V(s";
    command_vec.push((1, "passwd", "password:"));
    command_vec.push((2, new_password, "New password"));
    command_vec.push((3, new_password, "New password"));
    command_vec.push((4, new_password, "Retype new password"));
}
