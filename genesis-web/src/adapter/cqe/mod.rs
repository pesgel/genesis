use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use validator::Validate;

pub mod cmd;
pub mod query;
pub mod vo;

const RESPONSE_SUCCESS: &str = "success";
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
            msg: RESPONSE_SUCCESS.to_string(),
        }
    }
}

impl IntoResponse for ResponseSuccess {
    fn into_response(self) -> axum::response::Response {
        axum::Json(self).into_response()
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

impl<T> Response<T>
where
    T: Serialize,
{
    pub fn success(t: T) -> Response<T> {
        Self {
            code: 200,
            msg: RESPONSE_SUCCESS.to_string(),
            data: Some(t),
        }
    }
    pub fn failure(msg: String) -> Response<T> {
        Self {
            msg,
            code: 400,
            data: None,
        }
    }
}

impl<T> IntoResponse for Response<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        axum::Json(self).into_response()
    }
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

impl<T> IntoResponse for ResList<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        axum::Json(Response::success(self)).into_response()
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
