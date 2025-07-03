use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct GuacamoleConnParams {
    pub params: String,
    pub authorization: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct GuacamoleParams {
    // 屏幕宽
    #[validate(range(min = 1, message = "width is error"))]
    pub w: u16,
    // 屏幕高
    #[validate(range(min = 1, message = "height is error"))]
    pub h: u16,
    // dpi
    #[validate(range(min = 1, message = "dpi is error"))]
    pub dpi: u16,
    // 权限ID
    #[validate(length(min = 1, message = "permission is empty"))]
    pub permission_id: String,
}
