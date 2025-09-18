use super::defaults::*;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
#[derive(Debug, Default, Deserialize, Serialize, Builder, Clone)]
#[builder(setter(into))]
pub struct TargetSSHOptions {
    pub host: String,
    #[serde(default = "_default_ssh_port")]
    pub port: u16,
    #[serde(default = "_default_username")]
    pub username: String,
    #[serde(default)]
    #[builder(default)]
    pub allow_insecure_algos: Option<bool>,
    #[serde(default)]
    pub auth: SSHTargetAuth,
    #[serde(default)]
    pub pty_request: PtyRequest,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PtyRequest {
    pub term: String,
    pub width: u32,
    pub height: u32,
}

impl Default for PtyRequest {
    fn default() -> Self {
        Self {
            term: "xterm".to_string(),
            width: 80,
            height: 20,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum SSHTargetAuth {
    #[serde(rename = "password")]
    Password(SshTargetPasswordAuth),
    #[serde(rename = "publickey")]
    PublicKey(SshTargetPublicKeyAuth),
}

impl Default for SSHTargetAuth {
    fn default() -> Self {
        SSHTargetAuth::Password(SshTargetPasswordAuth::default())
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct SshTargetPasswordAuth {
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Default)]
pub struct SshTargetPublicKeyAuth {}
