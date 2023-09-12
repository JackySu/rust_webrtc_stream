#[allow(dead_code)]
use crate::{AppState, user::User};
use crate::api::api_response::ApiError;
use crate::token::{AuthorizationToken, SerializableToken};
use crate::JWT_SECRET;

use axum::response::IntoResponse;
use axum::{
    async_trait,
    extract::{FromRequestParts, State, Json},
    headers::{authorization::Bearer, Authorization, Cookie},
    http::request::Parts,
    TypedHeader,
};
use http::StatusCode;

pub struct JwtUserExtractor(pub AuthorizationToken);

#[async_trait]
impl FromRequestParts<AppState> for JwtUserExtractor {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Ok(TypedHeader(Authorization(jwt))) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state).await
        else {
            return Err(ApiError::AuthenticationRequired);
        };
        match AuthorizationToken::decode(jwt.token(), &JWT_SECRET) {
            Ok(v) => Ok(JwtUserExtractor(v)),
            Err(_) => Err(ApiError::AuthenticationExpired),
        }
    }
}

pub struct JwtWsUserExtractor(pub AuthorizationToken);

#[async_trait]
impl FromRequestParts<AppState> for JwtWsUserExtractor {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // extract token from query param or cookie
        let token = match parts.uri.query() {
            Some(query) => {
                let mut query = url::form_urlencoded::parse(query.as_bytes());
                let token = query.find(|(key, _)| key == "token");
                match token {
                    Some((_, token)) => token.into_owned(),
                    None => {
                        let Ok(TypedHeader(cookie)) = TypedHeader::<Cookie>::from_request_parts(parts, _state).await
                        else {
                            return Err(ApiError::AuthenticationRequired);
                        };
                        let cookie = cookie.get("token").ok_or(ApiError::AuthenticationRequired)?;
                        let cookie = cookie.split(";").collect::<Vec<&str>>();
                        let token = cookie
                            .iter()
                            .find(|cookie| cookie.starts_with("token"))
                            .ok_or(ApiError::AuthenticationRequired)?;
                        let token = token.split("=").collect::<Vec<&str>>();
                        token[1].to_string()
                    }
                }
            }
            None => return Err(ApiError::AuthenticationRequired),
        };
        match AuthorizationToken::decode(&token, &JWT_SECRET) {
            Ok(v) => Ok(JwtWsUserExtractor(v)),
            Err(_) => Err(ApiError::AuthenticationExpired),
        }
    }
}


#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AuthResponse {
    pub auth_token: String,
}

pub async fn auth_handler(State(_state): State<AppState>, Json(user): Json<User>) -> impl IntoResponse {
    let token = match AuthorizationToken::from(user).encode(&JWT_SECRET) {
        Ok(v) => v,
        Err(_) => return ApiError::TokenGenerationError.into_response(),
    };
    (StatusCode::OK, Json(AuthResponse { auth_token: token })).into_response()
}