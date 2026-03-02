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
    domains::file::{FileServiceTrait, dto::file_dto::UploadFileDto},
    domains::user::{
        UserRole, UserServiceTrait,
        dto::user_dto::{CreateUserMultipartDto, SearchUserDto, UpdateUserDto, UserDto},
    },
    web::web_routes,
};
use std::env;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[derive(Clone)]
struct TestState {
    auth_service: Arc<dyn AuthServiceTrait>,
    user_service: Arc<dyn UserServiceTrait>,
}

impl FromRef<TestState> for Arc<dyn AuthServiceTrait> {
    fn from_ref(input: &TestState) -> Self {
        Arc::clone(&input.auth_service)
    }
}

impl FromRef<TestState> for Arc<dyn UserServiceTrait> {
    fn from_ref(input: &TestState) -> Self {
        Arc::clone(&input.user_service)
    }
}

struct FakeAuthService;
struct FakeUserService;

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

#[async_trait]
impl UserServiceTrait for FakeUserService {
    fn create_service(
        _pool: sqlx::PgPool,
        _file_service: Arc<dyn FileServiceTrait>,
    ) -> Arc<dyn UserServiceTrait>
    where
        Self: Sized,
    {
        Arc::new(Self)
    }

    async fn get_user_by_id(&self, _id: Uuid) -> Result<UserDto, AppError> {
        Err(AppError::NotFound("not implemented".into()))
    }

    async fn get_user_list(
        &self,
        _search_user_dto: SearchUserDto,
    ) -> Result<Vec<UserDto>, AppError> {
        Ok(vec![])
    }

    async fn get_users(&self) -> Result<Vec<UserDto>, AppError> {
        Ok(vec![])
    }

    async fn create_user(
        &self,
        _create_user: CreateUserMultipartDto,
        _upload_file_dto: Option<&mut UploadFileDto>,
    ) -> Result<UserDto, AppError> {
        Err(AppError::InternalError)
    }

    async fn update_user(&self, _id: Uuid, _payload: UpdateUserDto) -> Result<UserDto, AppError> {
        Err(AppError::InternalError)
    }

    async fn delete_user(&self, _id: Uuid) -> Result<String, AppError> {
        Ok("ok".to_string())
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
        user_service: Arc::new(FakeUserService),
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

#[tokio::test]
async fn admin_users_page_forbidden_for_student_role() {
    ensure_jwt_env();
    let app = create_test_router();

    let token =
        jwt::make_jwt_token(&Uuid::new_v4(), UserRole::Student).expect("token should be created");

    let req = Request::builder()
        .method(Method::GET)
        .uri("/ui/admin/users")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request should build");

    let response = app.oneshot(req).await.expect("response should return");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn admin_users_page_allows_admin_role() {
    ensure_jwt_env();
    let app = create_test_router();

    let token =
        jwt::make_jwt_token(&Uuid::new_v4(), UserRole::Admin).expect("token should be created");

    let req = Request::builder()
        .method(Method::GET)
        .uri("/ui/admin/users")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request should build");

    let response = app.oneshot(req).await.expect("response should return");
    assert_eq!(response.status(), StatusCode::OK);
}
