use crate::config::SHARED_APP_CONFIG;
use crate::{
    error::{AppError, AuthError},
    util::jwt::{validate_jwt_token, Claims},
};
use axum::{body::Body, http::Request, middleware::Next, response::Response};
use serde::{Deserialize, Serialize};

const AUTHORIZATION_KEY: &str = "Authorization";
const AUTHORIZATION_SPLIT_KEY: &str = "Bearer";
pub async fn jwt_auth_middle(mut req: Request<Body>, next: Next) -> Result<Response, AppError> {
    let token = req
        .headers()
        .get(AUTHORIZATION_KEY)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| {
            value
                .strip_prefix(AUTHORIZATION_SPLIT_KEY)
                .map(|e| e.trim())
                .or(Some(value.trim()))
        })
        .or_else(|| {
            req.uri().query().and_then(|query| {
                query.split('&').find_map(|part| {
                    let (key, value) = part.split_once('=')?;
                    if key.to_uppercase() == AUTHORIZATION_KEY.to_uppercase() {
                        Some(value)
                    } else {
                        None
                    }
                })
            })
        });

    let token = token.ok_or(AuthError::MissingToken)?;
    let mut context = Context::default();

    context
        .with_claims(validate_jwt_token(
            token,
            SHARED_APP_CONFIG.read().await.jwt_config.secret.as_bytes(),
        )?)
        .with_token(token.to_string());

    req.extensions_mut().insert(context);
    Ok(next.run(req).await)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Context {
    pub token: String,
    pub claims: Claims,
}

impl Context {
    pub fn with_claims(&mut self, c: Claims) -> &mut Self {
        self.claims = c;
        self
    }

    pub fn with_token(&mut self, token: String) -> &mut Self {
        self.token = token;
        self
    }
}
