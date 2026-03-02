use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    Router,
    body::Body,
    extract::FromRef,
    http::{Method, Request, StatusCode},
};
use grade_o_matic::{
    common::{
        dto::RestApiResponse,
        error::AppError,
        jwt::{AuthBody, AuthPayload},
    },
    domains::auth::{AuthServiceTrait, dto::auth_dto::AuthUserDto, user_auth_routes},
};
use http_body_util::BodyExt;
use tower::ServiceExt;

const TEST_CLIENT_ID: &str = "apitest01";
const TEST_CLIENT_SECRET: &str = "test_password";
const TEST_USER_ID: &str = "00000000-0000-0000-0000-000000000000";

#[derive(Clone)]
struct TestState {
    auth_service: Arc<dyn AuthServiceTrait>,
}

impl FromRef<TestState> for Arc<dyn AuthServiceTrait> {
    fn from_ref(state: &TestState) -> Self {
        Arc::clone(&state.auth_service)
    }
}

struct FakeAuthService;

#[async_trait]
impl AuthServiceTrait for FakeAuthService {
    fn create_service(_pool: sqlx::PgPool) -> Arc<dyn AuthServiceTrait>
    where
        Self: Sized,
    {
        Arc::new(Self)
    }

    async fn create_user_auth(&self, _auth_user: AuthUserDto) -> Result<(), AppError> {
        Ok(())
    }

    async fn login_user(&self, auth_payload: AuthPayload) -> Result<AuthBody, AppError> {
        if auth_payload.client_id == TEST_CLIENT_ID
            && auth_payload.client_secret == TEST_CLIENT_SECRET
        {
            return Ok(AuthBody::new(format!("mock-token-for-{TEST_USER_ID}")));
        }

        if auth_payload.client_id == TEST_CLIENT_ID {
            return Err(AppError::WrongCredentials);
        }

        Err(AppError::UserNotFound)
    }
}

fn create_test_router() -> Router {
    let state = TestState {
        auth_service: Arc::new(FakeAuthService),
    };

    Router::new()
        .nest("/auth", user_auth_routes::<TestState>())
        .with_state(state)
}

async fn request_with_body<T: serde::Serialize>(
    method: Method,
    uri: &str,
    payload: &T,
) -> axum::response::Response {
    let json_payload = serde_json::to_string(payload).expect("Failed to serialize payload");
    let request: Request<Body> = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(json_payload))
        .unwrap();

    create_test_router().oneshot(request).await.unwrap()
}

async fn deserialize_json_body<T: serde::de::DeserializeOwned>(
    body: Body,
) -> Result<T, Box<dyn std::error::Error>> {
    let bytes = body.collect().await?.to_bytes();
    Ok(serde_json::from_slice::<T>(&bytes)?)
}

#[tokio::test]
async fn test_login_user() {
    let payload = AuthPayload {
        client_id: TEST_CLIENT_ID.to_string(),
        client_secret: TEST_CLIENT_SECRET.to_string(),
    };

    let response = request_with_body(Method::POST, "/auth/login", &payload).await;
    let (parts, body) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<AuthBody> = deserialize_json_body(body).await.unwrap();
    assert_eq!(response_body.0.status, StatusCode::OK.as_u16());

    let auth_body = response_body.0.data.unwrap();
    assert_eq!(auth_body.token_type, "Bearer");
    assert!(!auth_body.access_token.is_empty());
}

#[tokio::test]
async fn test_login_user_fail() {
    let payload = AuthPayload {
        client_id: TEST_CLIENT_ID.to_string(),
        client_secret: uuid::Uuid::new_v4().to_string(),
    };

    let response = request_with_body(Method::POST, "/auth/login", &payload).await;
    let (parts, body) = response.into_parts();

    assert_eq!(parts.status, StatusCode::UNAUTHORIZED);

    let response_body: RestApiResponse<()> = deserialize_json_body(body).await.unwrap();
    assert_eq!(response_body.0.status, StatusCode::UNAUTHORIZED.as_u16());
}

#[tokio::test]
async fn test_login_user_not_found() {
    let payload = AuthPayload {
        client_id: format!("testuser-{}", uuid::Uuid::new_v4()),
        client_secret: uuid::Uuid::new_v4().to_string(),
    };

    let response = request_with_body(Method::POST, "/auth/login", &payload).await;
    let (parts, body) = response.into_parts();

    assert_eq!(parts.status, StatusCode::NOT_FOUND);

    let response_body: RestApiResponse<()> = deserialize_json_body(body).await.unwrap();
    assert_eq!(response_body.0.status, StatusCode::NOT_FOUND.as_u16());
}
