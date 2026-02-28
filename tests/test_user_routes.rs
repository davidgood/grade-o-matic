use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use axum::{
    Router,
    body::Body,
    extract::{FromRef, Path},
    http::{Method, Request, StatusCode},
    response::IntoResponse,
    routing::delete,
};
use chrono::Utc;
use grade_o_matic::{
    common::{dto::RestApiResponse, error::AppError, jwt::Claims},
    domains::{
        file::dto::file_dto::UploadFileDto,
        user::{
            UserAssetPattern, UserServiceTrait,
            dto::user_dto::{CreateUserMultipartDto, SearchUserDto, UpdateUserDto, UserDto},
            user_routes,
        },
    },
};
use http_body_util::BodyExt;
use tokio::sync::Mutex;
use tower::ServiceExt;
use uuid::Uuid;

const TEST_USER_ID: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000000");

#[derive(Clone)]
struct TestState {
    user_service: Arc<dyn UserServiceTrait>,
    user_asset_pattern: UserAssetPattern,
}

impl FromRef<TestState> for Arc<dyn UserServiceTrait> {
    fn from_ref(state: &TestState) -> Self {
        Arc::clone(&state.user_service)
    }
}

impl FromRef<TestState> for UserAssetPattern {
    fn from_ref(state: &TestState) -> Self {
        state.user_asset_pattern.clone()
    }
}

#[derive(Default)]
struct UserStore {
    users: HashMap<Uuid, UserDto>,
}

struct FakeUserService {
    store: Arc<Mutex<UserStore>>,
}

impl FakeUserService {
    fn new() -> Self {
        let seed_id = Uuid::new_v4();
        let seed = UserDto {
            id: seed_id,
            username: "user0".to_string(),
            email: Some("user0@test.com".to_string()),
            created_by: Some(TEST_USER_ID),
            created_at: Some(Utc::now()),
            modified_by: Some(TEST_USER_ID),
            modified_at: Some(Utc::now()),
            file_id: None,
            origin_file_name: None,
        };

        let mut users = HashMap::new();
        users.insert(seed_id, seed);

        Self {
            store: Arc::new(Mutex::new(UserStore { users })),
        }
    }
}

#[async_trait]
impl UserServiceTrait for FakeUserService {
    fn create_service(
        _pool: sqlx::PgPool,
        _file_service: Arc<dyn grade_o_matic::domains::file::FileServiceTrait>,
    ) -> Arc<dyn UserServiceTrait>
    where
        Self: Sized,
    {
        Arc::new(Self::new())
    }

    async fn get_user_by_id(&self, id: Uuid) -> Result<UserDto, AppError> {
        let store = self.store.lock().await;
        store
            .users
            .get(&id)
            .cloned()
            .ok_or_else(|| AppError::NotFound("User not found".into()))
    }

    async fn get_user_list(
        &self,
        search_user_dto: SearchUserDto,
    ) -> Result<Vec<UserDto>, AppError> {
        let store = self.store.lock().await;
        let mut users: Vec<UserDto> = store.users.values().cloned().collect();

        if let Some(id) = search_user_dto.id {
            users.retain(|u| u.id == id);
        }
        if let Some(username) = search_user_dto.username {
            users.retain(|u| u.username.contains(&username));
        }
        if let Some(email) = search_user_dto.email {
            users.retain(|u| u.email.as_deref() == Some(email.as_str()));
        }

        Ok(users)
    }

    async fn get_users(&self) -> Result<Vec<UserDto>, AppError> {
        let store = self.store.lock().await;
        Ok(store.users.values().cloned().collect())
    }

    async fn create_user(
        &self,
        create_user: CreateUserMultipartDto,
        upload_file_dto: Option<&mut UploadFileDto>,
    ) -> Result<UserDto, AppError> {
        let now = Utc::now();
        let id = Uuid::new_v4();

        let mut user = UserDto {
            id,
            username: create_user.username,
            email: Some(create_user.email),
            created_by: Some(create_user.modified_by),
            created_at: Some(now),
            modified_by: Some(create_user.modified_by),
            modified_at: Some(now),
            file_id: None,
            origin_file_name: None,
        };

        if let Some(file) = upload_file_dto {
            user.file_id = Some(Uuid::new_v4().to_string());
            user.origin_file_name = Some(file.file.original_filename.clone());
            file.user_id = Some(id);
        }

        let mut store = self.store.lock().await;
        store.users.insert(id, user.clone());

        Ok(user)
    }

    async fn update_user(&self, id: Uuid, payload: UpdateUserDto) -> Result<UserDto, AppError> {
        let mut store = self.store.lock().await;
        let user = store
            .users
            .get_mut(&id)
            .ok_or_else(|| AppError::NotFound("User not found".into()))?;

        user.username = payload.username;
        user.email = Some(payload.email);
        user.modified_by = Some(payload.modified_by);
        user.modified_at = Some(Utc::now());

        Ok(user.clone())
    }

    async fn delete_user(&self, id: Uuid) -> Result<String, AppError> {
        let mut store = self.store.lock().await;
        if store.users.remove(&id).is_some() {
            Ok("User deleted".to_string())
        } else {
            Err(AppError::NotFound("User not found".into()))
        }
    }
}

async fn delete_file_stub(Path(_id): Path<String>) -> impl IntoResponse {
    RestApiResponse::success_with_message("File deleted", ())
}

fn create_test_router() -> Router {
    let state = TestState {
        user_service: Arc::new(FakeUserService::new()),
        user_asset_pattern: UserAssetPattern(
            regex::Regex::new(r"(?i)^.*\.(jpg|jpeg|png|gif|webp)$").unwrap(),
        ),
    };

    Router::new()
        .nest("/user", user_routes::<TestState>())
        .route("/file/{id}", delete(delete_file_stub))
        .with_state(state)
}

fn auth_claims() -> Claims {
    Claims {
        sub: TEST_USER_ID,
        ..Default::default()
    }
}

async fn deserialize_json_body<T: serde::de::DeserializeOwned>(
    body: Body,
) -> Result<T, Box<dyn std::error::Error>> {
    let bytes = body.collect().await?.to_bytes();
    Ok(serde_json::from_slice::<T>(&bytes)?)
}

async fn request_with_auth(app: &Router, method: Method, uri: &str) -> axum::response::Response {
    let mut req = Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    req.extensions_mut().insert(auth_claims());
    app.clone().oneshot(req).await.unwrap()
}

async fn request_with_auth_and_body<T: serde::Serialize>(
    app: &Router,
    method: Method,
    uri: &str,
    payload: &T,
) -> axum::response::Response {
    let json_payload = serde_json::to_string(payload).expect("Failed to serialize payload");
    let mut req = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(json_payload))
        .unwrap();
    req.extensions_mut().insert(auth_claims());
    app.clone().oneshot(req).await.unwrap()
}

async fn request_with_auth_and_multipart(
    app: &Router,
    method: Method,
    uri: &str,
    payload: Vec<u8>,
) -> axum::response::Response {
    let mut req = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "multipart/form-data; boundary=----XYZ")
        .body(Body::from(payload))
        .unwrap();
    req.extensions_mut().insert(auth_claims());
    app.clone().oneshot(req).await.unwrap()
}

async fn create_user(app: &Router) -> (CreateUserMultipartDto, UserDto) {
    let username = format!("testuser-{}", Uuid::new_v4());
    let email = format!("{}@test.com", username);

    let payload = CreateUserMultipartDto {
        username,
        email,
        modified_by: TEST_USER_ID,
        profile_picture: None,
    };

    let multipart_body = format!(
        "------XYZ\r\nContent-Disposition: form-data; name=\"username\"\r\n\r\n{}\r\n------XYZ\r\nContent-Disposition: form-data; name=\"email\"\r\n\r\n{}\r\n------XYZ--\r\n",
        payload.username, payload.email
    )
    .as_bytes()
    .to_vec();

    let response =
        request_with_auth_and_multipart(app, Method::POST, "/user", multipart_body).await;
    let (parts, body) = response.into_parts();
    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<UserDto> = deserialize_json_body(body).await.unwrap();
    (payload, response_body.0.data.unwrap())
}

async fn create_user_with_file(app: &Router) -> (CreateUserMultipartDto, UserDto, String) {
    let username = format!("testuser-{}", Uuid::new_v4());
    let email = format!("{}@test.com", username);
    let image_file = "cat.png";

    let payload = CreateUserMultipartDto {
        username,
        email,
        modified_by: TEST_USER_ID,
        profile_picture: Some(image_file.to_string()),
    };

    let file_bytes =
        std::fs::read(format!("tests/asset/{image_file}")).expect("failed reading test asset");

    let mut multipart_body = Vec::new();
    use std::io::Write;
    write!(
        &mut multipart_body,
        "------XYZ\r\nContent-Disposition: form-data; name=\"username\"\r\n\r\n{}\r\n",
        payload.username
    )
    .unwrap();
    write!(
        &mut multipart_body,
        "------XYZ\r\nContent-Disposition: form-data; name=\"email\"\r\n\r\n{}\r\n",
        payload.email
    )
    .unwrap();
    write!(
        &mut multipart_body,
        "------XYZ\r\nContent-Disposition: form-data; name=\"profile_picture\"; filename=\"{}\"\r\nContent-Type: image/png\r\n\r\n",
        image_file
    )
    .unwrap();
    multipart_body.extend_from_slice(&file_bytes);
    write!(&mut multipart_body, "\r\n------XYZ--\r\n").unwrap();

    let response =
        request_with_auth_and_multipart(app, Method::POST, "/user", multipart_body).await;
    let (parts, body) = response.into_parts();
    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<UserDto> = deserialize_json_body(body).await.unwrap();
    (
        payload,
        response_body.0.data.unwrap(),
        image_file.to_string(),
    )
}

#[tokio::test]
async fn test_create_user() {
    let app = create_test_router();
    let (payload, user_dto) = create_user(&app).await;

    assert!(!user_dto.id.is_nil());
    assert_eq!(user_dto.username, payload.username);
    assert_eq!(user_dto.email, Some(payload.email));
    assert_eq!(user_dto.modified_by, Some(TEST_USER_ID));
    assert_eq!(user_dto.origin_file_name, None);
    assert!(user_dto.file_id.is_none());
}

#[tokio::test]
async fn test_create_user_with_file() {
    let app = create_test_router();
    let (payload, user_dto, image_file) = create_user_with_file(&app).await;

    assert!(!user_dto.id.is_nil());
    assert_eq!(user_dto.username, payload.username);
    assert_eq!(user_dto.email, Some(payload.email));
    assert_eq!(user_dto.modified_by, Some(TEST_USER_ID));
    assert_eq!(user_dto.origin_file_name, Some(image_file));
    assert!(user_dto.file_id.is_some());
}

#[tokio::test]
async fn test_get_users() {
    let app = create_test_router();
    let response = request_with_auth(&app, Method::GET, "/user").await;
    let (parts, body) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<Vec<UserDto>> = deserialize_json_body(body).await.unwrap();
    let user_dtos = response_body.0.data.unwrap();
    assert!(!user_dtos.is_empty());
}

#[tokio::test]
async fn test_get_user_list() {
    let app = create_test_router();
    let payload = SearchUserDto {
        username: Some("user0".to_string()),
        id: None,
        email: None,
    };

    let response = request_with_auth_and_body(&app, Method::POST, "/user/list", &payload).await;
    let (parts, body) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<Vec<UserDto>> = deserialize_json_body(body).await.unwrap();
    let user_dtos = response_body.0.data.unwrap();
    assert!(!user_dtos.is_empty());
}

#[tokio::test]
async fn test_get_user_by_id() {
    let app = create_test_router();
    let (_, existent_user) = create_user(&app).await;
    let url = format!("/user/{}", existent_user.id);

    let response = request_with_auth(&app, Method::GET, &url).await;
    let (parts, body) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<UserDto> = deserialize_json_body(body).await.unwrap();
    let user_dto = response_body.0.data.unwrap();

    assert_eq!(user_dto.id, existent_user.id);
    assert_eq!(user_dto.username, existent_user.username);
    assert_eq!(user_dto.email, existent_user.email);
}

#[tokio::test]
async fn test_update_user() {
    let app = create_test_router();
    let (_, existent_user) = create_user(&app).await;

    let payload = UpdateUserDto {
        username: format!("update-testuser-{}", Uuid::new_v4()),
        email: format!("updated-{}@test.com", Uuid::new_v4()),
        modified_by: TEST_USER_ID,
    };

    let url = format!("/user/{}", existent_user.id);
    let response = request_with_auth_and_body(&app, Method::PUT, &url, &payload).await;
    let (parts, body) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<UserDto> = deserialize_json_body(body).await.unwrap();
    let user_dto = response_body.0.data.unwrap();

    assert_eq!(user_dto.id, existent_user.id);
    assert_eq!(user_dto.username, payload.username);
    assert_eq!(user_dto.email, Some(payload.email));
}

#[tokio::test]
async fn test_delete_user_not_found() {
    let app = create_test_router();
    let url = format!("/user/{}", Uuid::new_v4());
    let response = request_with_auth(&app, Method::DELETE, &url).await;
    let (parts, body) = response.into_parts();

    assert_eq!(parts.status, StatusCode::NOT_FOUND);

    let response_body: RestApiResponse<()> = deserialize_json_body(body).await.unwrap();
    assert_eq!(response_body.0.status, StatusCode::NOT_FOUND.as_u16());
}

#[tokio::test]
async fn test_delete_user() {
    let app = create_test_router();
    let (_, user) = create_user(&app).await;
    let url = format!("/user/{}", user.id);

    let response = request_with_auth(&app, Method::DELETE, &url).await;
    let (parts, body) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<()> = deserialize_json_body(body).await.unwrap();
    assert_eq!(response_body.0.status, StatusCode::OK.as_u16());
}

#[tokio::test]
async fn test_delete_user_file() {
    let app = create_test_router();
    let (_, user_dto, _) = create_user_with_file(&app).await;
    let file_id = user_dto.file_id.clone().unwrap_or_default();

    let url = format!("/file/{}", file_id);
    let response = request_with_auth(&app, Method::DELETE, &url).await;
    let (parts, body) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<()> = deserialize_json_body(body).await.unwrap();
    assert_eq!(response_body.0.status, StatusCode::OK.as_u16());
}
