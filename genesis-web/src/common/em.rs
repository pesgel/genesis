//! em

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub enum EnvelopeType {
    #[default]
    #[serde(rename = "r")]
    Raw,
    #[serde(rename = "w")]
    WindowSize,
}
