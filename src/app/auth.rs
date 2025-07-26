use crate::app::state::AppState;
use crate::config::config::Config;
use crate::core::error::AppError;
use async_trait::async_trait;
use axum::extract::{FromRequestParts};
use axum_extra::typed_header::TypedHeader;
use axum::http::request::Parts;
use axum::RequestPartsExt;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub username: String,
    pub exp: i64,
}

pub fn create_token(user_id: Uuid, username: &str, config: &Config) -> Result<String, AppError> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let claim = Claims {
        sub: user_id,
        username: username.to_owned(),
        exp: expiration
    };

    let jwt_secret = config.jwt_secret.as_bytes();

    encode(
        &Header::default(),
        &claim,
        &EncodingKey::from_secret(jwt_secret),
    ).map_err(|_| AppError::InternalError(String::from("JWT encoding error!")))
}

pub fn decode_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    let secret_key = DecodingKey::from_secret(secret.as_bytes());
    decode::<Claims>(&token, &secret_key, &Validation::default())
        .map(|data| data.claims)
        .map_err(|_| AppError::InternalError(String::from("JWT decode error!")))
}

pub struct AuthUser(pub Claims);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser where S: Send + Sync {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S)
        -> Result<Self, Self::Rejection>
    {
        let app_state = parts.extract_with_state::<AppState, S>(state).await.unwrap();

        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AppError::InternalError(String::from("TypedHeader error!")))?;
        let claims = decode_token(bearer.token(), &app_state.env.jwt_secret)?;
        Ok(AuthUser(claims))
    }
}
