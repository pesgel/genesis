use serde::{Deserialize, Serialize};
use validator::Validate;

pub mod cmd;
pub mod query;
pub mod vo;

// How we want errors responses to be serialized
#[derive(Clone, Serialize)]
pub struct ResponseSuccess {
    code: u16,
    msg: String,
}

impl Default for ResponseSuccess {
    fn default() -> Self {
        Self {
            code: 200,
            msg: "success".to_string(),
        }
    }
}

#[derive(Clone, Serialize)]
pub struct Response<T>
where
    T: Serialize,
{
    pub code: u16,
    pub msg: String,
    pub data: Option<T>,
}

#[derive(Clone, Serialize)]
pub struct ResList<T>
where
    T: Serialize,
{
    list: Vec<T>,
    total: u64,
}

impl<T> ResList<T>
where
    T: Serialize,
{
    pub fn new(total: u64, list: Vec<T>) -> ResList<T> {
        Self { total, list }
    }
}

impl<T> Response<T>
where
    T: Serialize,
{
    pub fn new_success(t: T) -> Response<T> {
        Self {
            code: 200,
            msg: "success".to_string(),
            data: Some(t),
        }
    }
    pub fn new_failure(msg: String) -> Response<T> {
        Self {
            msg,
            code: 400,
            data: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteReplaceItem {
    #[validate(length(min = 1, message = "execute replace item value is empty"))]
    pub value: String,
    #[validate(length(min = 1, message = "execute replace item mark is empty"))]
    pub mark: String,
    #[validate(length(min = 1, message = "execute replace item replace type is empty"))]
    pub replace_type: String,
}
