use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserVO {
    pub id: String,
    pub name: String,
    pub username: String,
    pub phone: String,
    pub email: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginRes {
    pub token: String,
}
