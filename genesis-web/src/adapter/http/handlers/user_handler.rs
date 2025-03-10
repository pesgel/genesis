use crate::adapter::cmd::user::{UserLoginCmd, UserRegisterCmd};
use crate::adapter::http::middleware::auth::Context;
use crate::adapter::vo::user::{LoginRes, UserVO};
use crate::adapter::{Response, ResponseSuccess};
use crate::config::{AppState, SHARED_APP_CONFIG};
use crate::error::{AppError, AppJson};
use crate::repo::model::user;
use crate::repo::sea::{SeaRepo, UserRepo};
use crate::util::jwt;
use axum::extract::{Path, State};
use axum::{Extension, Json};
use uuid::Uuid;

pub async fn user_info(
    State(state): State<AppState>,
    Extension(ctx): Extension<Context>,
) -> Result<Json<Response<UserVO>>, AppError> {
    let user: user::Model = UserRepo::get_user_by_id(&state.conn, &ctx.claims.user_id).await?;
    Ok(Json(Response::new_success(UserVO {
        id: user.id,
        name: user.name,
        username: user.username,
        phone: user.phone,
        email: user.email,
    })))
}
pub async fn user_login(
    State(state): State<AppState>,
    AppJson(data): AppJson<UserLoginCmd>,
) -> Result<Json<Response<LoginRes>>, AppError> {
    let user: user::Model = UserRepo::find_user_by_username(&state.conn, &data.username).await?;
    if user.password != data.password {
        return Err(AppError::MsgError(
            "username or password is error".to_string(),
        ));
    };
    let config = SHARED_APP_CONFIG.read().await;
    let mut cla = jwt::Claims::new(
        config.jwt_config.expire_time,
        config.jwt_config.issuer.clone(),
    )?;
    let token = cla
        .with_name(user.name.clone())
        .with_username(user.username.clone())
        .with_email(user.email.clone())
        .with_phone(user.phone.clone())
        .with_user_id(user.id.clone())
        .generate_token(config.jwt_config.secret.as_bytes())?;
    let user_vo = LoginRes { token };
    Ok(Json(Response::new_success(user_vo)))
}

pub async fn user_register(
    State(state): State<AppState>,
    AppJson(data): AppJson<UserRegisterCmd>,
) -> Result<Json<ResponseSuccess>, AppError> {
    let user = user::Model {
        id: Uuid::new_v4().to_string(),
        name: data.name,
        username: data.username,
        password: data.password,
        email: data.email,
        phone: data.phone,
        remark: data.remark,
        ..Default::default()
    };
    UserRepo::insert_user_one(&state.conn, user).await?;
    Ok(Json(ResponseSuccess::default()))
}

pub async fn delete_user_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ResponseSuccess>, AppError> {
    SeaRepo::delete_by_id::<user::Entity>(&state.conn, &id)
        .await
        .map(|_| Ok(Json(ResponseSuccess::default())))?
}
