use async_trait::async_trait;
use axum::{
    Router,
    body::Body,
    extract::FromRef,
    http::{
        Method, Request, StatusCode,
        header::{AUTHORIZATION, LOCATION},
    },
};
use grade_o_matic::{
    common::error::AppError,
    common::jwt::{self, AuthBody, AuthPayload},
    domains::auth::AuthServiceTrait,
    domains::auth::dto::auth_dto::AuthUserDto,
    domains::user::UserRole,
    web::web_routes,
};
use std::env;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[derive(Clone)]
struct TestState {
    auth_service: Arc<dyn AuthServiceTrait>,
}

impl FromRef<TestState> for Arc<dyn AuthServiceTrait> {
    fn from_ref(input: &TestState) -> Self {
        Arc::clone(&input.auth_service)
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

    async fn login_user(&self, _auth_payload: AuthPayload) -> Result<AuthBody, AppError> {
        Err(AppError::WrongCredentials)
    }
}

fn ensure_jwt_env() {
    if env::var("JWT_SECRET_KEY").is_err() {
        unsafe {
            env::set_var("JWT_SECRET_KEY", "ci-test-jwt-secret");
        }
    }
}

fn create_test_router() -> Router {
    let state = TestState {
        auth_service: Arc::new(FakeAuthService),
    };

    Router::new()
        .merge(web_routes::<TestState>())
        .with_state(state)
}

#[tokio::test]
async fn ui_assignments_requires_authentication() {
    ensure_jwt_env();
    let app = create_test_router();

    let req = Request::builder()
        .method(Method::GET)
        .uri("/ui/assignments")
        .body(Body::empty())
        .expect("request should build");

    let response = app.oneshot(req).await.expect("response should return");
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response
            .headers()
            .get(LOCATION)
            .and_then(|v| v.to_str().ok()),
        Some("/ui/login")
    );
}

#[tokio::test]
async fn ui_assignments_allows_authenticated_user_role() {
    ensure_jwt_env();
    let app = create_test_router();

    let token =
        jwt::make_jwt_token(&Uuid::new_v4(), UserRole::Student).expect("token should be created");

    let req = Request::builder()
        .method(Method::GET)
        .uri("/ui/assignments")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request should build");

    let response = app.oneshot(req).await.expect("response should return");
    assert_eq!(response.status(), StatusCode::OK);
}
