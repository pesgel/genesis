//! em

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, FromRepr};

#[derive(Serialize, Deserialize, Debug, Default)]
pub enum EnvelopeType {
    #[default]
    #[serde(rename = "r")]
    Raw,
    #[serde(rename = "w")]
    WindowSize,
}

#[derive(Serialize, Clone, Deserialize, Debug, Default, FromRepr, AsRefStr)]
pub enum AssetType {
    // 服务器
    #[default]
    #[serde(rename = "servers")]
    Servers,
    // 容器
    #[serde(rename = "container")]
    Container,
    // 数据库
    #[serde(rename = "database")]
    Database,
    // 应用
    #[serde(rename = "app")]
    Application,
}

#[derive(Serialize, Clone, Deserialize, Debug, Default, FromRepr, AsRefStr)]
pub enum AssetAddressType {
    #[default]
    #[serde(rename = "ip")]
    IP,
    #[serde(rename = "uri")]
    URI,
    #[serde(rename = "domain")]
    Domain,
}

#[derive(Serialize, Clone, Deserialize, Debug, Default, FromRepr, AsRefStr)]
pub enum AuthType {
    #[default]
    #[serde(rename = "password")]
    Password,
    #[serde(rename = "certificateStr")]
    CertificateStr,
    #[serde(rename = "certificatePath")]
    CertificatePath,
}

#[derive(Serialize, Clone, Deserialize, Debug, Default, FromRepr, AsRefStr)]
pub enum AssetProtocolType {
    #[serde(rename = "db")]
    DB,
    #[default]
    #[serde(rename = "auto")]
    AUTO,
    #[serde(rename = "ssh")]
    SSH,
    #[serde(rename = "rdp")]
    RDP,
    #[serde(rename = "vnc")]
    VNC,
    #[serde(rename = "app")]
    APP,
    #[serde(rename = "telnet")]
    TELNET,
}
