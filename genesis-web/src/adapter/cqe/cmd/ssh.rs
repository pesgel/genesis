use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SSHConnParams {
    pub params: String,
    pub authorization: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ConnParams {
    // 屏幕宽
    #[validate(range(min = 1, message = "width is error"))]
    pub w: u8,
    // 屏幕高
    #[validate(range(min = 1, message = "height is error"))]
    pub h: u8,
    // terminal 模式
    #[validate(length(min = 1, message = "term is empty"))]
    pub term: String,
    // 权限ID
    #[validate(length(min = 1, message = "permission is empty"))]
    pub opt_permission_id: String,
}
