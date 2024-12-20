use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UserLoginCmd {
    #[validate(length(min = 1, message = "name is empty"))]
    pub username: String,
    #[validate(length(min = 1, message = "password is empty"))]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UserRegisterCmd {
    #[validate(length(min = 1, message = "username is empty"))]
    pub username: String,
    #[validate(length(min = 1, message = "password is empty"))]
    pub password: String,
    #[validate(length(min = 1, message = "phone is empty"))]
    pub phone: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub remark: String,
    #[serde(default)]
    pub name: String,
}
