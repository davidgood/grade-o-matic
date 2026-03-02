use axum::{
    extract::Request,
    http::header,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};

use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use std::{env, fmt::Display};
use utoipa::ToSchema;

use super::error::AppError;
use crate::domains::user::UserRole;

/// JWT_SECRET_KEY is the environment variable that holds the secret key for JWT encoding and decoding.
/// It is loaded from the environment variables using the dotenv crate.
/// The secret key is used to sign the JWT tokens and should be kept secret.
pub static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    dotenvy::dotenv().ok();

    let secret = env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY must be set");
    Keys::new(secret.as_bytes())
});

/// Keys is a struct that holds the encoding and decoding keys for JWT.
pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

/// The Keys struct is used to create the encoding and decoding keys for JWT.
impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

/// Claims is a struct that represents the claims in the JWT token.
/// It contains the subject (user ID), expiration time, and issued at time.
/// The `sub` field is the user ID, `exp` is the expiration time, and `iat` is the issued at time.
/// The `Claims` struct is used to encode and decode the JWT tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: uuid::Uuid,
    pub user_role: UserRole,
    pub exp: usize,
    pub iat: usize,
}

/// The Claims struct implements the `Display` trait for easy printing.
/// It formats the claims as a string, showing the user ID.
impl Display for Claims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "user_id: {}, role: {:?}", self.sub, self.user_role)
    }
}

/// The Default trait is implemented for the Claims struct.
/// It sets the default values for the claims.
impl Default for Claims {
    fn default() -> Self {
        let now = Utc::now();
        let expire: Duration = Duration::hours(24);
        let exp: usize = (now + expire).timestamp() as usize;
        let iat: usize = now.timestamp() as usize;
        Claims {
            sub: uuid::Uuid::new_v4(),
            user_role: UserRole::Student,
            exp,
            iat,
        }
    }
}

/// AuthBody is a struct that represents the authentication body.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuthBody {
    pub access_token: String,
    pub token_type: String,
}

/// The AuthBody struct is used to create a new instance of the authentication body.
/// It takes an access token as a parameter and sets the token type to "Bearer".
impl AuthBody {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}

/// AuthPayload is a struct that represents the authentication payload.
/// It contains the client ID and client secret.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuthPayload {
    pub client_id: String,
    pub client_secret: String,
}

/// make_jwt_token is a function that creates a JWT token.
/// It takes a user ID as a parameter and returns a Result with the JWT token or an error.
pub fn make_jwt_token(user_id: &uuid::Uuid, user_role: UserRole) -> Result<String, AppError> {
    let claims = Claims {
        sub: *user_id,
        user_role,
        ..Default::default()
    };
    encode(&Header::default(), &claims, &KEYS.encoding).map_err(|_| AppError::TokenCreation)
}

/// Middleware to validate JWT tokens.
/// If the token is valid, the request proceeds; otherwise, a 401 Unauthorized is returned.
pub async fn jwt_auth<B>(mut req: Request<B>, next: Next) -> Result<Response, Response>
where
    B: Send + Into<axum::body::Body>,
{
    let token = extract_token_from_headers(req.headers())
        .ok_or_else(|| AppError::InvalidToken.into_response())?;

    // Validate and decode the token.
    let claims = decode_token(token).map_err(|_| AppError::InvalidToken.into_response())?;

    // Insert the decoded claims into the request extensions.
    req.extensions_mut().insert(claims);
    Ok(next.run(req.map(Into::into)).await)
}

/// Middleware to enforce web UI access by role.
pub async fn require_ui_access<B>(req: Request<B>, next: Next) -> Result<Response, Response>
where
    B: Send + Into<axum::body::Body>,
{
    let claims = req
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| AppError::InvalidToken.into_response())?;

    if !can_access_ui(&claims.user_role) {
        return Err(AppError::Forbidden.into_response());
    }

    Ok(next.run(req.map(Into::into)).await)
}

/// Role policy for web UI routes.
pub fn can_access_ui(role: &UserRole) -> bool {
    matches!(
        role,
        UserRole::Admin | UserRole::Instructor | UserRole::Ta | UserRole::Student
    )
}

/// Middleware to validate JWT for browser UI routes.
/// Reads bearer token from Authorization header or `auth_token` cookie.
/// Redirects to `/ui/login` when token is missing/invalid.
pub async fn jwt_auth_web<B>(mut req: Request<B>, next: Next) -> Result<Response, Response>
where
    B: Send + Into<axum::body::Body>,
{
    let token = extract_token_from_headers(req.headers())
        .ok_or_else(|| Redirect::to("/ui/login").into_response())?;

    let claims = decode_token(token).map_err(|_| Redirect::to("/ui/login").into_response())?;

    req.extensions_mut().insert(claims);
    Ok(next.run(req.map(Into::into)).await)
}

fn decode_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(token, &KEYS.decoding, &Validation::default()).map(|data| data.claims)
}

fn extract_token_from_headers(headers: &header::HeaderMap) -> Option<&str> {
    extract_bearer_token(headers).or_else(|| extract_auth_cookie_token(headers))
}

fn extract_bearer_token(headers: &header::HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|t| !t.is_empty())
}

fn extract_auth_cookie_token(headers: &header::HeaderMap) -> Option<&str> {
    let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;

    cookie_header
        .split(';')
        .map(str::trim)
        .find_map(|kv| kv.strip_prefix("auth_token="))
        .filter(|t| !t.is_empty())
}
