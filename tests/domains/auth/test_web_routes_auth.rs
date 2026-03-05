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
use chrono::Utc;
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
        ClassMembershipRole, ClassMembershipServiceTrait,
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

fn enrolled_class_id() -> Uuid {
    Uuid::parse_str("11111111-1111-1111-1111-111111111111").expect("valid uuid")
}

fn other_class_id() -> Uuid {
    Uuid::parse_str("22222222-2222-2222-2222-222222222222").expect("valid uuid")
}

fn instructor_owner_id() -> Uuid {
    Uuid::parse_str("33333333-3333-3333-3333-333333333333").expect("valid uuid")
}

fn other_instructor_id() -> Uuid {
    Uuid::parse_str("44444444-4444-4444-4444-444444444444").expect("valid uuid")
}

fn instructor_owned_class_id() -> Uuid {
    Uuid::parse_str("55555555-5555-5555-5555-555555555555").expect("valid uuid")
}

fn instructor_unowned_class_id() -> Uuid {
    Uuid::parse_str("66666666-6666-6666-6666-666666666666").expect("valid uuid")
}

fn instructor_assignment_id() -> Uuid {
    Uuid::parse_str("77777777-7777-7777-7777-777777777777").expect("valid uuid")
}

fn roster_membership_id() -> Uuid {
    Uuid::parse_str("88888888-8888-8888-8888-888888888888").expect("valid uuid")
}

fn mismatched_roster_membership_id() -> Uuid {
    Uuid::parse_str("99999999-9999-9999-9999-999999999999").expect("valid uuid")
}

fn enrolled_student_id() -> Uuid {
    Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").expect("valid uuid")
}

fn available_student_id() -> Uuid {
    Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").expect("valid uuid")
}

fn ta_user_id() -> Uuid {
    Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").expect("valid uuid")
}

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
        let mk_user = |id: Uuid, username: &str, email: &str, role: UserRole| UserDto {
            id,
            username: username.to_string(),
            email: Some(email.to_string()),
            created_by: None,
            created_at: Some(Utc::now()),
            modified_by: None,
            modified_at: Some(Utc::now()),
            file_id: None,
            origin_file_name: None,
            user_role: role,
        };

        if _id == enrolled_student_id() {
            return Ok(mk_user(
                enrolled_student_id(),
                "student01",
                "student01@example.com",
                UserRole::Student,
            ));
        }
        if _id == available_student_id() {
            return Ok(mk_user(
                available_student_id(),
                "student02",
                "student02@example.com",
                UserRole::Student,
            ));
        }
        if _id == ta_user_id() {
            return Ok(mk_user(
                ta_user_id(),
                "ta01",
                "ta01@example.com",
                UserRole::Ta,
            ));
        }

        Err(AppError::NotFound("not implemented".into()))
    }

    async fn get_user_list(
        &self,
        _search_user_dto: SearchUserDto,
    ) -> Result<Vec<UserDto>, AppError> {
        Ok(vec![])
    }

    async fn get_users(&self) -> Result<Vec<UserDto>, AppError> {
        Ok(vec![
            UserDto {
                id: enrolled_student_id(),
                username: "student01".to_string(),
                email: Some("student01@example.com".to_string()),
                created_by: None,
                created_at: Some(Utc::now()),
                modified_by: None,
                modified_at: Some(Utc::now()),
                file_id: None,
                origin_file_name: None,
                user_role: UserRole::Student,
            },
            UserDto {
                id: available_student_id(),
                username: "student02".to_string(),
                email: Some("student02@example.com".to_string()),
                created_by: None,
                created_at: Some(Utc::now()),
                modified_by: None,
                modified_at: Some(Utc::now()),
                file_id: None,
                origin_file_name: None,
                user_role: UserRole::Student,
            },
            UserDto {
                id: ta_user_id(),
                username: "ta01".to_string(),
                email: Some("ta01@example.com".to_string()),
                created_by: None,
                created_at: Some(Utc::now()),
                modified_by: None,
                modified_at: Some(Utc::now()),
                file_id: None,
                origin_file_name: None,
                user_role: UserRole::Ta,
            },
        ])
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
        if _class_id == enrolled_class_id() {
            return Ok(vec![AssignmentDto {
                id: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").expect("valid uuid"),
                class_id: enrolled_class_id(),
                title: "Homework 1".to_string(),
                description: Some("Ownership and borrowing".to_string()),
                due_at: None,
                points: Some(100),
            }]);
        }

        if _class_id == other_class_id() {
            return Ok(vec![AssignmentDto {
                id: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").expect("valid uuid"),
                class_id: other_class_id(),
                title: "Hidden Homework".to_string(),
                description: Some("Should never be visible to this student".to_string()),
                due_at: None,
                points: Some(50),
            }]);
        }

        Ok(vec![])
    }

    async fn list_by_class_with_attachment_count(
        &self,
        _class_id: Uuid,
    ) -> Result<Vec<AssignmentWithAttachmentCountDto>, AppError> {
        if _class_id == instructor_owned_class_id() {
            return Ok(vec![AssignmentWithAttachmentCountDto {
                id: instructor_assignment_id(),
                class_id: instructor_owned_class_id(),
                title: "Midterm Project".to_string(),
                description: Some("Build a service".to_string()),
                due_at: None,
                points: Some(200),
                attachment_count: 0,
            }]);
        }
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
        if _id == instructor_assignment_id() {
            return Ok(Some(AssignmentDto {
                id: instructor_assignment_id(),
                class_id: instructor_owned_class_id(),
                title: "Midterm Project".to_string(),
                description: Some("Build a service".to_string()),
                due_at: None,
                points: Some(200),
            }));
        }
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
        Ok(AssignmentDto {
            id: _assignment.id,
            class_id: _assignment.class_id,
            title: _assignment.title,
            description: _assignment.description,
            due_at: _assignment.due_at,
            points: _assignment.points,
        })
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
        Ok(vec![
            ClassDto {
                id: instructor_owned_class_id(),
                title: "Systems Programming".to_string(),
                description: Some("Owned by instructor".to_string()),
                term: Some("Spring 2026".to_string()),
                owner_id: Some(instructor_owner_id()),
                created_at: Some(Utc::now()),
            },
            ClassDto {
                id: instructor_unowned_class_id(),
                title: "Algorithms".to_string(),
                description: Some("Owned by a different instructor".to_string()),
                term: Some("Spring 2026".to_string()),
                owner_id: Some(other_instructor_id()),
                created_at: Some(Utc::now()),
            },
        ])
    }

    async fn list_classes_with_assignments(
        &self,
        _owner_id: Uuid,
    ) -> Result<
        Vec<grade_o_matic::domains::classes::dto::class_dto::ClassesWithAssignmentsDto>,
        AppError,
    > {
        if _owner_id == instructor_owner_id() {
            return Ok(vec![
                grade_o_matic::domains::classes::dto::class_dto::ClassesWithAssignmentsDto {
                    class_id: instructor_owned_class_id(),
                    class_title: "Systems Programming".to_string(),
                    class_term: Some("Spring 2026".to_string()),
                    assignment_id: Some(instructor_assignment_id()),
                    assignment_title: Some("Midterm Project".to_string()),
                    assignment_description: Some("Build a service".to_string()),
                    due_at: None,
                    points: Some(200),
                },
            ]);
        }

        Ok(vec![])
    }

    async fn find_by_id(&self, _id: Uuid) -> Result<Option<ClassDto>, AppError> {
        if _id == instructor_owned_class_id() {
            return Ok(Some(ClassDto {
                id: instructor_owned_class_id(),
                title: "Systems Programming".to_string(),
                description: Some("Owned by instructor".to_string()),
                term: Some("Spring 2026".to_string()),
                owner_id: Some(instructor_owner_id()),
                created_at: Some(Utc::now()),
            }));
        }

        if _id == instructor_unowned_class_id() {
            return Ok(Some(ClassDto {
                id: instructor_unowned_class_id(),
                title: "Algorithms".to_string(),
                description: Some("Owned by a different instructor".to_string()),
                term: Some("Spring 2026".to_string()),
                owner_id: Some(other_instructor_id()),
                created_at: Some(Utc::now()),
            }));
        }

        if _id == enrolled_class_id() {
            return Ok(Some(ClassDto {
                id: enrolled_class_id(),
                title: "Intro to Rust".to_string(),
                description: Some("Foundations of Rust programming".to_string()),
                term: Some("Spring 2026".to_string()),
                owner_id: Some(Uuid::new_v4()),
                created_at: Some(Utc::now()),
            }));
        }

        if _id == other_class_id() {
            return Ok(Some(ClassDto {
                id: other_class_id(),
                title: "Distributed Systems".to_string(),
                description: Some("Hidden class".to_string()),
                term: Some("Fall 2026".to_string()),
                owner_id: Some(Uuid::new_v4()),
                created_at: Some(Utc::now()),
            }));
        }

        Ok(None)
    }

    async fn create(
        &self,
        _class: grade_o_matic::domains::classes::dto::class_dto::CreateClassDto,
    ) -> Result<ClassDto, AppError> {
        Ok(ClassDto {
            id: Uuid::parse_str("12121212-1212-1212-1212-121212121212").expect("valid uuid"),
            title: _class.title,
            description: _class.description,
            term: _class.term,
            owner_id: _class.owner_id.or(Some(_class.modified_by)),
            created_at: Some(Utc::now()),
        })
    }

    async fn update(
        &self,
        _class: grade_o_matic::domains::classes::dto::class_dto::UpdateClassDto,
    ) -> Result<Option<ClassDto>, AppError> {
        if _class.id == instructor_owned_class_id() {
            return Ok(Some(ClassDto {
                id: _class.id,
                title: _class.title,
                description: _class.description,
                term: _class.term,
                owner_id: _class.owner_id,
                created_at: Some(Utc::now()),
            }));
        }
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
        if _class_id == instructor_owned_class_id() {
            return Ok(vec![ClassMembershipDto {
                id: roster_membership_id(),
                class_id: instructor_owned_class_id(),
                user_id: enrolled_student_id(),
                role: ClassMembershipRole::Student,
                created_at: Some(Utc::now()),
                modified_at: Some(Utc::now()),
            }]);
        }
        Ok(vec![])
    }

    async fn list_by_user_id(&self, _user_id: Uuid) -> Result<Vec<ClassMembershipDto>, AppError> {
        Ok(vec![ClassMembershipDto {
            id: Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").expect("valid uuid"),
            class_id: enrolled_class_id(),
            user_id: _user_id,
            role: ClassMembershipRole::Student,
            created_at: Some(Utc::now()),
            modified_at: Some(Utc::now()),
        }])
    }

    async fn find_by_id(&self, _id: Uuid) -> Result<Option<ClassMembershipDto>, AppError> {
        if _id == roster_membership_id() {
            return Ok(Some(ClassMembershipDto {
                id: roster_membership_id(),
                class_id: instructor_owned_class_id(),
                user_id: enrolled_student_id(),
                role: ClassMembershipRole::Student,
                created_at: Some(Utc::now()),
                modified_at: Some(Utc::now()),
            }));
        }
        if _id == mismatched_roster_membership_id() {
            return Ok(Some(ClassMembershipDto {
                id: mismatched_roster_membership_id(),
                class_id: instructor_unowned_class_id(),
                user_id: enrolled_student_id(),
                role: ClassMembershipRole::Student,
                created_at: Some(Utc::now()),
                modified_at: Some(Utc::now()),
            }));
        }
        Ok(None)
    }

    async fn create(
        &self,
        _membership: CreateClassMembershipDto,
    ) -> Result<ClassMembershipDto, AppError> {
        Ok(ClassMembershipDto {
            id: Uuid::parse_str("13131313-1313-1313-1313-131313131313").expect("valid uuid"),
            class_id: _membership.class_id,
            user_id: _membership.user_id,
            role: _membership.role,
            created_at: Some(Utc::now()),
            modified_at: Some(Utc::now()),
        })
    }

    async fn update(
        &self,
        _membership: UpdateClassMembershipDto,
    ) -> Result<Option<ClassMembershipDto>, AppError> {
        Ok(None)
    }

    async fn delete(&self, _id: Uuid) -> Result<String, AppError> {
        Ok("Class membership deleted".to_string())
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

async fn get_csrf_cookie_and_token(app: &Router) -> (String, String) {
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
    (csrf_cookie, authenticity_token)
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

    let token = jwt::make_jwt_token(&Uuid::new_v4(), UserRole::Instructor)
        .expect("token should be created");

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
async fn ui_assignments_forbidden_for_student_role() {
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
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn students_classes_allows_student_role() {
    ensure_jwt_env();
    let app = create_test_router();

    let token =
        jwt::make_jwt_token(&Uuid::new_v4(), UserRole::Student).expect("token should be created");

    let req = Request::builder()
        .method(Method::GET)
        .uri("/ui/students/classes")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request should build");

    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("response should return");
    assert_eq!(response.status(), StatusCode::OK);

    let html = body_to_string(response.into_body()).await;
    assert!(html.contains("Intro to Rust"));
    assert!(!html.contains("Distributed Systems"));
}

#[tokio::test]
async fn students_classes_forbidden_for_instructor_role() {
    ensure_jwt_env();
    let app = create_test_router();

    let token = jwt::make_jwt_token(&Uuid::new_v4(), UserRole::Instructor)
        .expect("token should be created");

    let req = Request::builder()
        .method(Method::GET)
        .uri("/ui/students/classes")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request should build");

    let response = app.oneshot(req).await.expect("response should return");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn students_assignments_forbidden_for_admin_role() {
    ensure_jwt_env();
    let app = create_test_router();

    let token =
        jwt::make_jwt_token(&Uuid::new_v4(), UserRole::Admin).expect("token should be created");

    let req = Request::builder()
        .method(Method::GET)
        .uri("/ui/students/assignments")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request should build");

    let response = app.oneshot(req).await.expect("response should return");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn students_assignments_allows_student_role_and_scopes_results() {
    ensure_jwt_env();
    let app = create_test_router();

    let token =
        jwt::make_jwt_token(&Uuid::new_v4(), UserRole::Student).expect("token should be created");

    let req = Request::builder()
        .method(Method::GET)
        .uri("/ui/students/assignments")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request should build");

    let response = app.oneshot(req).await.expect("response should return");
    assert_eq!(response.status(), StatusCode::OK);

    let html = body_to_string(response.into_body()).await;
    assert!(html.contains("Homework 1"));
    assert!(html.contains("Intro to Rust"));
    assert!(!html.contains("Hidden Homework"));
}

#[tokio::test]
async fn instructors_classes_allows_instructor_and_scopes_to_owned_classes() {
    ensure_jwt_env();
    let app = create_test_router();

    let token = jwt::make_jwt_token(&instructor_owner_id(), UserRole::Instructor)
        .expect("token should be created");

    let req = Request::builder()
        .method(Method::GET)
        .uri("/ui/instructors/classes")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request should build");

    let response = app.oneshot(req).await.expect("response should return");
    assert_eq!(response.status(), StatusCode::OK);

    let html = body_to_string(response.into_body()).await;
    assert!(html.contains("Systems Programming"));
    assert!(!html.contains("Algorithms"));
}

#[tokio::test]
async fn instructor_class_detail_forbidden_for_non_owner_instructor() {
    ensure_jwt_env();
    let app = create_test_router();

    let token = jwt::make_jwt_token(&other_instructor_id(), UserRole::Instructor)
        .expect("token should be created");

    let req = Request::builder()
        .method(Method::GET)
        .uri("/ui/instructors/classes/55555555-5555-5555-5555-555555555555")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request should build");

    let response = app.oneshot(req).await.expect("response should return");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn instructor_assignments_fragment_renders_for_instructor() {
    ensure_jwt_env();
    let app = create_test_router();

    let token = jwt::make_jwt_token(&instructor_owner_id(), UserRole::Instructor)
        .expect("token should be created");

    let req = Request::builder()
        .method(Method::GET)
        .uri("/ui/instructors/fragments/assignments/table")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request should build");

    let response = app.oneshot(req).await.expect("response should return");
    assert_eq!(response.status(), StatusCode::OK);
    let html = body_to_string(response.into_body()).await;
    assert!(html.contains("Midterm Project"));
    assert!(html.contains("Systems Programming"));
}

#[tokio::test]
async fn instructor_assignments_fragment_forbidden_for_student() {
    ensure_jwt_env();
    let app = create_test_router();

    let token =
        jwt::make_jwt_token(&Uuid::new_v4(), UserRole::Student).expect("token should be created");

    let req = Request::builder()
        .method(Method::GET)
        .uri("/ui/instructors/fragments/assignments/table")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request should build");

    let response = app.oneshot(req).await.expect("response should return");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn create_class_page_allows_instructor_role() {
    ensure_jwt_env();
    let app = create_test_router();

    let token = jwt::make_jwt_token(&instructor_owner_id(), UserRole::Instructor)
        .expect("token should be created");

    let req = Request::builder()
        .method(Method::GET)
        .uri("/ui/instructors/classes/new")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request should build");

    let response = app.oneshot(req).await.expect("response should return");
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn create_class_submit_happy_path_redirects_to_class_detail() {
    ensure_jwt_env();
    let app = create_test_router();
    let (csrf_cookie, authenticity_token) = get_csrf_cookie_and_token(&app).await;
    let encoded_token = url_encode(&authenticity_token);

    let token = jwt::make_jwt_token(&instructor_owner_id(), UserRole::Instructor)
        .expect("token should be created");

    let body = format!(
        "title=Operating%20Systems&description=Kernel%20project&term=Spring%202026&authenticity_token={encoded_token}"
    );
    let req = Request::builder()
        .method(Method::POST)
        .uri("/ui/instructors/classes/new")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .header(COOKIE, csrf_cookie)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .expect("request should build");

    let response = app.oneshot(req).await.expect("response should return");
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let location = response
        .headers()
        .get(LOCATION)
        .and_then(|v| v.to_str().ok())
        .expect("location should exist");
    assert!(location.starts_with("/ui/instructors/classes/"));
}

#[tokio::test]
async fn create_class_submit_rejects_empty_title() {
    ensure_jwt_env();
    let app = create_test_router();
    let (csrf_cookie, authenticity_token) = get_csrf_cookie_and_token(&app).await;
    let encoded_token = url_encode(&authenticity_token);

    let token = jwt::make_jwt_token(&instructor_owner_id(), UserRole::Instructor)
        .expect("token should be created");

    let body =
        format!("title=%20%20%20&description=desc&term=Spring&authenticity_token={encoded_token}");
    let req = Request::builder()
        .method(Method::POST)
        .uri("/ui/instructors/classes/new")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .header(COOKIE, csrf_cookie)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .expect("request should build");

    let response = app.oneshot(req).await.expect("response should return");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let html = body_to_string(response.into_body()).await;
    assert!(html.contains("Title is required."));
}

#[tokio::test]
async fn add_student_to_roster_happy_path_redirects_back_to_class() {
    ensure_jwt_env();
    let app = create_test_router();
    let (csrf_cookie, authenticity_token) = get_csrf_cookie_and_token(&app).await;
    let encoded_token = url_encode(&authenticity_token);

    let token = jwt::make_jwt_token(&instructor_owner_id(), UserRole::Instructor)
        .expect("token should be created");

    let body = format!(
        "student_user_id={}&authenticity_token={encoded_token}",
        available_student_id()
    );
    let req = Request::builder()
        .method(Method::POST)
        .uri("/ui/instructors/classes/55555555-5555-5555-5555-555555555555/roster")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .header(COOKIE, csrf_cookie)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .expect("request should build");

    let response = app.oneshot(req).await.expect("response should return");
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response
            .headers()
            .get(LOCATION)
            .and_then(|v| v.to_str().ok()),
        Some("/ui/instructors/classes/55555555-5555-5555-5555-555555555555")
    );
}

#[tokio::test]
async fn add_student_to_roster_rejects_non_student_user() {
    ensure_jwt_env();
    let app = create_test_router();
    let (csrf_cookie, authenticity_token) = get_csrf_cookie_and_token(&app).await;
    let encoded_token = url_encode(&authenticity_token);

    let token = jwt::make_jwt_token(&instructor_owner_id(), UserRole::Instructor)
        .expect("token should be created");

    let body = format!(
        "student_user_id={}&authenticity_token={encoded_token}",
        ta_user_id()
    );
    let req = Request::builder()
        .method(Method::POST)
        .uri("/ui/instructors/classes/55555555-5555-5555-5555-555555555555/roster")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .header(COOKIE, csrf_cookie)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .expect("request should build");

    let response = app.oneshot(req).await.expect("response should return");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn remove_student_from_roster_forbidden_when_membership_class_mismatch() {
    ensure_jwt_env();
    let app = create_test_router();
    let (csrf_cookie, authenticity_token) = get_csrf_cookie_and_token(&app).await;
    let encoded_token = url_encode(&authenticity_token);

    let token = jwt::make_jwt_token(&instructor_owner_id(), UserRole::Instructor)
        .expect("token should be created");

    let body = format!("authenticity_token={encoded_token}");
    let req = Request::builder()
        .method(Method::POST)
        .uri("/ui/instructors/classes/55555555-5555-5555-5555-555555555555/roster/99999999-9999-9999-9999-999999999999/delete")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .header(COOKIE, csrf_cookie)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .expect("request should build");

    let response = app.oneshot(req).await.expect("response should return");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn edit_assignment_page_happy_path_renders_form() {
    ensure_jwt_env();
    let app = create_test_router();

    let token = jwt::make_jwt_token(&instructor_owner_id(), UserRole::Instructor)
        .expect("token should be created");

    let req = Request::builder()
        .method(Method::GET)
        .uri("/ui/instructors/assignments/77777777-7777-7777-7777-777777777777/edit")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request should build");

    let response = app.oneshot(req).await.expect("response should return");
    assert_eq!(response.status(), StatusCode::OK);
    let html = body_to_string(response.into_body()).await;
    assert!(html.contains("Edit Assignment"));
    assert!(html.contains("Midterm Project"));
}

#[tokio::test]
async fn edit_assignment_submit_happy_path_redirects_to_class() {
    ensure_jwt_env();
    let app = create_test_router();
    let (csrf_cookie, authenticity_token) = get_csrf_cookie_and_token(&app).await;
    let encoded_token = url_encode(&authenticity_token);

    let token = jwt::make_jwt_token(&instructor_owner_id(), UserRole::Instructor)
        .expect("token should be created");

    let body = format!(
        "title=Updated%20Project&description=Updated%20desc&due_at=2026-03-31T23:59&points=123&authenticity_token={encoded_token}"
    );
    let req = Request::builder()
        .method(Method::POST)
        .uri("/ui/instructors/assignments/77777777-7777-7777-7777-777777777777/edit")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .header(COOKIE, csrf_cookie)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .expect("request should build");

    let response = app.oneshot(req).await.expect("response should return");
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response
            .headers()
            .get(LOCATION)
            .and_then(|v| v.to_str().ok()),
        Some("/ui/instructors/classes/55555555-5555-5555-5555-555555555555")
    );
}

#[tokio::test]
async fn edit_assignment_submit_rejects_invalid_due_date() {
    ensure_jwt_env();
    let app = create_test_router();
    let (csrf_cookie, authenticity_token) = get_csrf_cookie_and_token(&app).await;
    let encoded_token = url_encode(&authenticity_token);

    let token = jwt::make_jwt_token(&instructor_owner_id(), UserRole::Instructor)
        .expect("token should be created");

    let body = format!(
        "title=Updated%20Project&description=Updated%20desc&due_at=bad-date&points=123&authenticity_token={encoded_token}"
    );
    let req = Request::builder()
        .method(Method::POST)
        .uri("/ui/instructors/assignments/77777777-7777-7777-7777-777777777777/edit")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .header(COOKIE, csrf_cookie)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .expect("request should build");

    let response = app.oneshot(req).await.expect("response should return");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
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
