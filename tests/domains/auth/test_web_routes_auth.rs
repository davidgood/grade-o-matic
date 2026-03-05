use async_trait::async_trait;
use axum::{
    Router,
    body::Body,
    extract::FromRef,
    http::{
        Method, Request, StatusCode,
        header::{AUTHORIZATION, CONTENT_TYPE, COOKIE, LOCATION, SET_COOKIE},
    },
};
use grade_o_matic::{
    common::error::AppError,
    common::jwt::{self, AuthBody, AuthPayload},
    domains::assignments::{
        AssignmentAttachment, AssignmentServiceTrait,
        dto::assignment_dto::{AssignmentDto, AssignmentWithAttachmentCountDto},
    },
    domains::auth::AuthServiceTrait,
    domains::auth::dto::auth_dto::AuthUserDto,
    domains::class_memberships::{
        ClassMembershipServiceTrait,
        dto::class_membership_dto::{
            ClassMembershipDto, CreateClassMembershipDto, UpdateClassMembershipDto,
        },
    },
    domains::classes::{ClassServiceTrait, dto::class_dto::ClassDto},
    domains::file::{FileServiceTrait, dto::file_dto::UploadFileDto},
    domains::user::{
        UserAssetPattern, UserRole, UserServiceTrait,
        dto::user_dto::{CreateUserMultipartDto, SearchUserDto, UpdateUserDto, UserDto},
    },
    web::web_routes,
};
use http_body_util::BodyExt;
use std::env;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[derive(Clone)]
struct TestState {
    auth_service: Arc<dyn AuthServiceTrait>,
    assignment_service: Arc<dyn AssignmentServiceTrait>,
    class_membership_service: Arc<dyn ClassMembershipServiceTrait>,
    class_service: Arc<dyn ClassServiceTrait>,
    file_service: Arc<dyn FileServiceTrait>,
    user_asset_pattern: UserAssetPattern,
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

impl FromRef<TestState> for Arc<dyn AssignmentServiceTrait> {
    fn from_ref(input: &TestState) -> Self {
        Arc::clone(&input.assignment_service)
    }
}

impl FromRef<TestState> for Arc<dyn ClassServiceTrait> {
    fn from_ref(input: &TestState) -> Self {
        Arc::clone(&input.class_service)
    }
}

impl FromRef<TestState> for Arc<dyn ClassMembershipServiceTrait> {
    fn from_ref(input: &TestState) -> Self {
        Arc::clone(&input.class_membership_service)
    }
}

impl FromRef<TestState> for Arc<dyn FileServiceTrait> {
    fn from_ref(input: &TestState) -> Self {
        Arc::clone(&input.file_service)
    }
}

impl FromRef<TestState> for UserAssetPattern {
    fn from_ref(input: &TestState) -> Self {
        input.user_asset_pattern.clone()
    }
}

struct FakeAuthService;
struct FakeAssignmentService;
struct FakeClassService;
struct FakeClassMembershipService;
struct FakeUserService;
struct FakeFileService;

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

#[async_trait]
impl AssignmentServiceTrait for FakeAssignmentService {
    fn create_service(_pool: sqlx::PgPool) -> Arc<dyn AssignmentServiceTrait>
    where
        Self: Sized,
    {
        Arc::new(Self)
    }

    async fn list(&self) -> Result<Vec<AssignmentDto>, AppError> {
        Ok(vec![])
    }

    async fn list_by_class(&self, _class_id: Uuid) -> Result<Vec<AssignmentDto>, AppError> {
        Ok(vec![])
    }

    async fn list_by_class_with_attachment_count(
        &self,
        _class_id: Uuid,
    ) -> Result<Vec<AssignmentWithAttachmentCountDto>, AppError> {
        Ok(vec![])
    }

    async fn list_attachments(
        &self,
        _assignment_id: Uuid,
    ) -> Result<Vec<AssignmentAttachment>, AppError> {
        Ok(vec![])
    }

    async fn attach_file(
        &self,
        _assignment_id: Uuid,
        _file_id: Uuid,
        _created_by: Uuid,
    ) -> Result<(), AppError> {
        Ok(())
    }

    async fn remove_file(&self, _assignment_id: Uuid, _file_id: Uuid) -> Result<bool, AppError> {
        Ok(true)
    }

    async fn find_by_id(&self, _id: Uuid) -> Result<Option<AssignmentDto>, AppError> {
        Ok(None)
    }

    async fn create(
        &self,
        _assignment: grade_o_matic::domains::assignments::dto::assignment_dto::CreateAssignmentDto,
    ) -> Result<AssignmentDto, AppError> {
        Err(AppError::InternalError)
    }

    async fn update(
        &self,
        _assignment: grade_o_matic::domains::assignments::dto::assignment_dto::UpdateAssignmentDto,
    ) -> Result<AssignmentDto, AppError> {
        Err(AppError::InternalError)
    }

    async fn delete(&self, _id: Uuid) -> Result<String, AppError> {
        Ok("ok".to_string())
    }
}

#[async_trait]
impl FileServiceTrait for FakeFileService {
    fn create_service(
        _config: grade_o_matic::common::config::Config,
        _pool: sqlx::PgPool,
    ) -> Arc<dyn FileServiceTrait>
    where
        Self: Sized,
    {
        Arc::new(Self)
    }

    async fn process_profile_picture_upload(
        &self,
        _tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        _upload_file_dto: &UploadFileDto,
    ) -> Result<Option<grade_o_matic::domains::file::dto::file_dto::UploadedFileDto>, AppError>
    {
        Ok(None)
    }

    async fn process_assignment_file_upload(
        &self,
        _upload_file_dto: &UploadFileDto,
    ) -> Result<grade_o_matic::domains::file::dto::file_dto::UploadedFileDto, AppError> {
        Err(AppError::InternalError)
    }

    async fn get_file_metadata(
        &self,
        _file_id: Uuid,
    ) -> Result<Option<grade_o_matic::domains::file::dto::file_dto::UploadedFileDto>, AppError>
    {
        Ok(None)
    }

    async fn delete_file(&self, _file_id: Uuid) -> Result<String, AppError> {
        Ok("ok".to_string())
    }
}

#[async_trait]
impl ClassServiceTrait for FakeClassService {
    fn create_class_service(_pool: sqlx::PgPool) -> Arc<dyn ClassServiceTrait>
    where
        Self: Sized,
    {
        Arc::new(Self)
    }

    async fn list(&self) -> Result<Vec<ClassDto>, AppError> {
        Ok(vec![])
    }

    async fn list_classes_with_assignments(
        &self,
        _owner_id: Uuid,
    ) -> Result<
        Vec<grade_o_matic::domains::classes::dto::class_dto::ClassesWithAssignmentsDto>,
        AppError,
    > {
        Ok(vec![])
    }

    async fn find_by_id(&self, _id: Uuid) -> Result<Option<ClassDto>, AppError> {
        Ok(None)
    }

    async fn create(
        &self,
        _class: grade_o_matic::domains::classes::dto::class_dto::CreateClassDto,
    ) -> Result<ClassDto, AppError> {
        Err(AppError::InternalError)
    }

    async fn update(
        &self,
        _class: grade_o_matic::domains::classes::dto::class_dto::UpdateClassDto,
    ) -> Result<Option<ClassDto>, AppError> {
        Ok(None)
    }

    async fn delete(&self, _id: Uuid) -> Result<String, AppError> {
        Ok("ok".to_string())
    }
}

#[async_trait]
impl ClassMembershipServiceTrait for FakeClassMembershipService {
    fn create_service(_pool: sqlx::PgPool) -> Arc<dyn ClassMembershipServiceTrait>
    where
        Self: Sized,
    {
        Arc::new(Self)
    }

    async fn list_by_class_id(&self, _class_id: Uuid) -> Result<Vec<ClassMembershipDto>, AppError> {
        Ok(vec![])
    }

    async fn list_by_user_id(&self, _user_id: Uuid) -> Result<Vec<ClassMembershipDto>, AppError> {
        Ok(vec![])
    }

    async fn find_by_id(&self, _id: Uuid) -> Result<Option<ClassMembershipDto>, AppError> {
        Ok(None)
    }

    async fn create(
        &self,
        _membership: CreateClassMembershipDto,
    ) -> Result<ClassMembershipDto, AppError> {
        Err(AppError::InternalError)
    }

    async fn update(
        &self,
        _membership: UpdateClassMembershipDto,
    ) -> Result<Option<ClassMembershipDto>, AppError> {
        Ok(None)
    }

    async fn delete(&self, _id: Uuid) -> Result<String, AppError> {
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
        assignment_service: Arc::new(FakeAssignmentService),
        class_membership_service: Arc::new(FakeClassMembershipService),
        class_service: Arc::new(FakeClassService),
        file_service: Arc::new(FakeFileService),
        user_asset_pattern: UserAssetPattern(
            regex::Regex::new(r"(?i)^.*\.(jpg|jpeg|png|gif|webp|pdf)$")
                .expect("regex should compile"),
        ),
        user_service: Arc::new(FakeUserService),
    };

    Router::new()
        .merge(web_routes::<TestState>())
        .with_state(state)
}

async fn body_to_string(body: Body) -> String {
    let bytes = body
        .collect()
        .await
        .expect("body should collect")
        .to_bytes();
    String::from_utf8(bytes.to_vec()).expect("body should be valid utf-8")
}

fn extract_csrf_cookie(headers: &axum::http::HeaderMap) -> Option<String> {
    headers.get_all(SET_COOKIE).iter().find_map(|value| {
        let raw = value.to_str().ok()?;
        raw.split(';')
            .next()
            .filter(|cookie| cookie.starts_with("Csrf_Token="))
            .map(str::to_string)
    })
}

fn extract_hidden_authenticity_token(html: &str) -> Option<String> {
    let marker = "name=\"authenticity_token\" value=\"";
    let start = html.find(marker)?;
    let rest = &html[start + marker.len()..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn url_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for b in value.bytes() {
        let is_unreserved = b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~');
        if is_unreserved {
            out.push(char::from(b));
        } else {
            out.push('%');
            out.push_str(&format!("{b:02X}"));
        }
    }
    out
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

#[tokio::test]
async fn login_page_issues_csrf_cookie_and_hidden_token() {
    ensure_jwt_env();
    let app = create_test_router();

    let req = Request::builder()
        .method(Method::GET)
        .uri("/ui/login")
        .body(Body::empty())
        .expect("request should build");

    let response = app.oneshot(req).await.expect("response should return");
    assert_eq!(response.status(), StatusCode::OK);

    let csrf_cookie = extract_csrf_cookie(response.headers());
    assert!(csrf_cookie.is_some(), "csrf cookie should be set");

    let html = body_to_string(response.into_body()).await;
    let authenticity_token = extract_hidden_authenticity_token(&html);
    assert!(
        authenticity_token.is_some(),
        "authenticity token input should be present"
    );
}

#[tokio::test]
async fn login_submit_rejects_invalid_csrf_token() {
    ensure_jwt_env();
    let app = create_test_router();

    let req = Request::builder()
        .method(Method::POST)
        .uri("/ui/login")
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(
            "username=admin01&password=test_password&authenticity_token=invalid",
        ))
        .expect("request should build");

    let response = app.oneshot(req).await.expect("response should return");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn login_submit_with_valid_csrf_reaches_auth_handler() {
    ensure_jwt_env();
    let app = create_test_router();

    let get_req = Request::builder()
        .method(Method::GET)
        .uri("/ui/login")
        .body(Body::empty())
        .expect("request should build");
    let get_response = app
        .clone()
        .oneshot(get_req)
        .await
        .expect("response should return");

    let csrf_cookie =
        extract_csrf_cookie(get_response.headers()).expect("csrf cookie should exist");
    let html = body_to_string(get_response.into_body()).await;
    let authenticity_token =
        extract_hidden_authenticity_token(&html).expect("token should exist in form");
    let encoded_token = url_encode(&authenticity_token);

    let post_body =
        format!("username=admin01&password=test_password&authenticity_token={encoded_token}");
    let post_req = Request::builder()
        .method(Method::POST)
        .uri("/ui/login")
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(COOKIE, csrf_cookie)
        .body(Body::from(post_body))
        .expect("request should build");

    let post_response = app.oneshot(post_req).await.expect("response should return");
    // FakeAuthService always returns WrongCredentials, so valid CSRF should pass through to auth
    // and produce UNAUTHORIZED (not FORBIDDEN).
    assert_eq!(post_response.status(), StatusCode::UNAUTHORIZED);
}
