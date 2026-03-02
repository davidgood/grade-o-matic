use crate::test_helpers::{TEST_USER_ID, deserialize_json_body};
use async_trait::async_trait;
use axum::Router;
use axum::body::Body;
use axum::extract::FromRef;
use axum::http::{Method, Request, StatusCode};
use chrono::Utc;
use grade_o_matic::common::dto::RestApiResponse;
use grade_o_matic::common::error::AppError;
use grade_o_matic::common::jwt::Claims;
use grade_o_matic::domains::classes::dto::class_dto::{CreateClassDto, UpdateClassDto};
use grade_o_matic::domains::classes::{ClassServiceTrait, class_routes, dto::class_dto::ClassDto};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceExt;
use uuid::Uuid;

#[derive(Clone)]
struct TestState {
    class_service: Arc<dyn ClassServiceTrait>,
}

impl FromRef<TestState> for Arc<dyn ClassServiceTrait> {
    fn from_ref(state: &TestState) -> Self {
        Arc::clone(&state.class_service)
    }
}

#[derive(Default)]
struct ClassStore {
    classes: HashMap<Uuid, ClassDto>,
}
struct FakeClassService {
    store: Arc<Mutex<ClassStore>>,
}

impl FakeClassService {
    fn new() -> Self {
        let seed_id = Uuid::new_v4();
        let seed = ClassDto {
            id: seed_id,
            title: "Seed Class".to_string(),
            description: Some("This is a seed class for testing".to_string()),
            created_at: Some(Utc::now()),
        };

        let mut classes = HashMap::new();
        classes.insert(seed_id, seed);

        Self {
            store: Arc::new(Mutex::new(ClassStore { classes })),
        }
    }
}

#[async_trait]
impl ClassServiceTrait for FakeClassService {
    fn create_class_service(_pool: PgPool) -> Arc<dyn ClassServiceTrait>
    where
        Self: Sized,
    {
        Arc::new(Self::new())
    }

    async fn list(&self) -> Result<Vec<ClassDto>, AppError> {
        let store = self.store.lock().await;
        Ok(store.classes.values().cloned().collect())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<ClassDto>, AppError> {
        let store = self.store.lock().await;
        match store.classes.get(&id).cloned() {
            Some(class) => Ok(Some(class)),
            None => Err(AppError::NotFound("Class not found".into())),
        }
    }

    async fn create(&self, class: CreateClassDto) -> Result<ClassDto, AppError> {
        let class = ClassDto {
            id: Uuid::new_v4(),
            title: class.title,
            description: class.description,
            created_at: Some(Utc::now()),
        };

        let mut store = self.store.lock().await;
        store.classes.insert(class.id, class.clone());
        Ok(class)
    }

    async fn update(&self, payload: UpdateClassDto) -> Result<Option<ClassDto>, AppError> {
        let mut store = self.store.lock().await;

        let class = store
            .classes
            .get_mut(&payload.id)
            .ok_or_else(|| AppError::NotFound("Class not found".into()))?;

        class.title = payload.title.clone();
        class.description = payload.description.clone();

        Ok(Some(class.clone()))
    }

    async fn delete(&self, id: Uuid) -> Result<String, AppError> {
        let mut store = self.store.lock().await;
        if store.classes.remove(&id).is_some() {
            Ok("Class deleted".to_string())
        } else {
            Err(AppError::NotFound("Class not found".into()))
        }
    }
}

fn create_test_router() -> axum::Router {
    let state = TestState {
        class_service: Arc::new(FakeClassService::new()),
    };

    Router::new()
        .nest("/classes", class_routes::<TestState>())
        .with_state(state)
}

fn auth_claims() -> Claims {
    Claims {
        sub: TEST_USER_ID,
        ..Default::default()
    }
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

async fn create_class(app: &Router) -> (CreateClassDto, ClassDto) {
    let payload = CreateClassDto {
        title: "Test Class".to_string(),
        description: Some("This is a test class".to_string()),
        modified_by: TEST_USER_ID,
    };

    let response = request_with_auth_and_body(&app, Method::POST, "/classes", &payload).await;

    assert_eq!(response.status(), StatusCode::CREATED);
    let response_body: RestApiResponse<ClassDto> =
        deserialize_json_body(response.into_body()).await.unwrap();

    assert_eq!(response_body.0.status, StatusCode::CREATED.as_u16());

    let class_dto = response_body.0.data.unwrap();
    (payload, class_dto)
}

#[tokio::test]
async fn test_get_classes() {
    let app = create_test_router();
    let response = request_with_auth(&app, Method::GET, "/classes").await;
    let (parts, body) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);
    let response_body: RestApiResponse<Vec<ClassDto>> = deserialize_json_body(body).await.unwrap();
    let classes = response_body.0.data.unwrap();
    assert!(!classes.is_empty());
}

#[tokio::test]
async fn test_get_class_by_id() {
    let app = create_test_router();
    let (_, class) = create_class(&app).await;

    let url = format!("/classes/{}", class.id);
    let response = request_with_auth(&app, Method::GET, &url).await;
    assert_eq!(response.status(), StatusCode::OK);
    let response_body: RestApiResponse<ClassDto> =
        deserialize_json_body(response.into_body()).await.unwrap();
    let class_dto = response_body.0.data.unwrap();
    assert_eq!(class.id, class_dto.id);
    assert_eq!(class.title, class_dto.title);
    assert_eq!(class.description, class_dto.description);
    assert_eq!(class.created_at, class_dto.created_at);
}

#[tokio::test]
async fn test_create_class() {
    let app = create_test_router();
    let (payload, class_dto) = create_class(&app).await;

    assert_eq!(payload.title, class_dto.title);
    assert_eq!(payload.description, class_dto.description);
}

#[tokio::test]
async fn test_update_class() {
    let app = create_test_router();
    let (_, class) = create_class(&app).await;

    let payload = UpdateClassDto {
        id: class.id,
        title: "Updated Title".to_string(),
        description: Some("Updated Description".to_string()),
        modified_by: TEST_USER_ID,
    };

    let response = request_with_auth_and_body(&app, Method::PUT, "/classes", &payload).await;
    assert_eq!(response.status(), StatusCode::OK);

    let response_body: RestApiResponse<ClassDto> =
        deserialize_json_body(response.into_body()).await.unwrap();
    let class_dto = response_body.0.data.unwrap();

    assert_eq!(payload.title, class_dto.title);
    assert_eq!(payload.description, class_dto.description);
}

#[tokio::test]
async fn test_delete_class() {
    let app = create_test_router();
    let (_, class) = create_class(&app).await;

    let url = format!("/classes/{}", class.id);
    let response = request_with_auth(&app, Method::DELETE, &url).await;

    assert_eq!(response.status(), StatusCode::OK);

    let response_body: RestApiResponse<String> =
        deserialize_json_body(response.into_body()).await.unwrap();

    assert_eq!(response_body.0.status, StatusCode::OK.as_u16());
    assert_eq!(response_body.0.message, "Class deleted");
}
