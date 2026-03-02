use crate::test_helpers::{TEST_USER_ID, deserialize_json_body};
use async_trait::async_trait;
use axum::Router;
use axum::body::Body;
use axum::extract::FromRef;
use axum::http::{Method, Request, StatusCode};
use chrono::Utc;
use grade_o_matic::common::dto::RestApiResponse;
use grade_o_matic::common::error::AppError;
use grade_o_matic::domains::class_memberships::dto::class_membership_dto::{
    ClassMembershipDto, CreateClassMembershipDto, UpdateClassMembershipDto,
};
use grade_o_matic::domains::class_memberships::{
    ClassMembershipRole, ClassMembershipServiceTrait, class_membership_routes,
};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceExt;
use uuid::Uuid;

#[derive(Clone)]
struct TestState {
    class_membership_service: Arc<dyn ClassMembershipServiceTrait>,
}

impl FromRef<TestState> for Arc<dyn ClassMembershipServiceTrait> {
    fn from_ref(state: &TestState) -> Self {
        Arc::clone(&state.class_membership_service)
    }
}

#[derive(Default)]
struct MembershipStore {
    memberships: HashMap<Uuid, ClassMembershipDto>,
}

struct FakeClassMembershipService {
    store: Arc<Mutex<MembershipStore>>,
}

impl FakeClassMembershipService {
    fn new() -> Self {
        let seed_id = Uuid::new_v4();
        let seed = ClassMembershipDto {
            id: seed_id,
            class_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            role: ClassMembershipRole::Student,
            created_at: Some(Utc::now()),
            modified_at: Some(Utc::now()),
        };

        let mut memberships = HashMap::new();
        memberships.insert(seed_id, seed);
        Self {
            store: Arc::new(Mutex::new(MembershipStore { memberships })),
        }
    }
}

#[async_trait]
impl ClassMembershipServiceTrait for FakeClassMembershipService {
    fn create_service(_pool: PgPool) -> Arc<dyn ClassMembershipServiceTrait>
    where
        Self: Sized,
    {
        Arc::new(Self::new())
    }

    async fn list_by_class_id(&self, class_id: Uuid) -> Result<Vec<ClassMembershipDto>, AppError> {
        let store = self.store.lock().await;
        Ok(store
            .memberships
            .values()
            .filter(|m| m.class_id == class_id)
            .cloned()
            .collect())
    }

    async fn list_by_user_id(&self, user_id: Uuid) -> Result<Vec<ClassMembershipDto>, AppError> {
        let store = self.store.lock().await;
        Ok(store
            .memberships
            .values()
            .filter(|m| m.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<ClassMembershipDto>, AppError> {
        let store = self.store.lock().await;
        Ok(store.memberships.get(&id).cloned())
    }

    async fn create(
        &self,
        membership: CreateClassMembershipDto,
    ) -> Result<ClassMembershipDto, AppError> {
        let dto = ClassMembershipDto {
            id: Uuid::new_v4(),
            class_id: membership.class_id,
            user_id: membership.user_id,
            role: membership.role,
            created_at: Some(Utc::now()),
            modified_at: Some(Utc::now()),
        };

        let mut store = self.store.lock().await;
        store.memberships.insert(dto.id, dto.clone());
        Ok(dto)
    }

    async fn update(
        &self,
        membership: UpdateClassMembershipDto,
    ) -> Result<Option<ClassMembershipDto>, AppError> {
        let mut store = self.store.lock().await;
        let Some(existing) = store.memberships.get_mut(&membership.id) else {
            return Ok(None);
        };
        existing.role = membership.role;
        existing.modified_at = Some(Utc::now());
        Ok(Some(existing.clone()))
    }

    async fn delete(&self, id: Uuid) -> Result<String, AppError> {
        let mut store = self.store.lock().await;
        if store.memberships.remove(&id).is_some() {
            Ok("Class membership deleted".to_string())
        } else {
            Err(AppError::NotFound("Class membership not found".to_string()))
        }
    }
}

fn create_test_router() -> Router {
    let state = TestState {
        class_membership_service: Arc::new(FakeClassMembershipService::new()),
    };

    Router::new()
        .nest("/class-memberships", class_membership_routes::<TestState>())
        .with_state(state)
}

async fn request_with_body<T: serde::Serialize>(
    app: &Router,
    method: Method,
    uri: &str,
    payload: &T,
) -> axum::response::Response {
    let json_payload = serde_json::to_string(payload).expect("Failed to serialize payload");
    let req = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(json_payload))
        .unwrap();
    app.clone().oneshot(req).await.unwrap()
}

async fn request(app: &Router, method: Method, uri: &str) -> axum::response::Response {
    let req = Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    app.clone().oneshot(req).await.unwrap()
}

fn payload_from_json<T: serde::de::DeserializeOwned>(value: serde_json::Value) -> T {
    serde_json::from_value(value).expect("failed to deserialize payload")
}

#[tokio::test]
async fn test_create_and_get_class_membership() {
    let app = create_test_router();

    let create_payload: CreateClassMembershipDto = payload_from_json(serde_json::json!({
        "class_id": Uuid::new_v4(),
        "user_id": TEST_USER_ID,
        "role": "ta"
    }));

    let create_response =
        request_with_body(&app, Method::POST, "/class-memberships", &create_payload).await;

    assert_eq!(create_response.status(), StatusCode::CREATED);
    let create_body: RestApiResponse<ClassMembershipDto> =
        deserialize_json_body(create_response.into_body())
            .await
            .unwrap();
    let created = create_body.0.data.unwrap();
    assert!(matches!(created.role, ClassMembershipRole::Ta));

    let get_url = format!("/class-memberships/{}", created.id);
    let get_response = request(&app, Method::GET, &get_url).await;
    assert_eq!(get_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_by_user_id_returns_memberships() {
    let app = create_test_router();
    let create_payload: CreateClassMembershipDto = payload_from_json(serde_json::json!({
        "class_id": Uuid::new_v4(),
        "user_id": TEST_USER_ID,
        "role": "student"
    }));

    let create_response =
        request_with_body(&app, Method::POST, "/class-memberships", &create_payload).await;
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let list_url = format!("/class-memberships/user/{TEST_USER_ID}");
    let list_response = request(&app, Method::GET, &list_url).await;
    assert_eq!(list_response.status(), StatusCode::OK);

    let list_body: RestApiResponse<Vec<ClassMembershipDto>> =
        deserialize_json_body(list_response.into_body())
            .await
            .unwrap();
    let memberships = list_body.0.data.unwrap();
    assert!(!memberships.is_empty());
}

#[tokio::test]
async fn test_update_and_delete_class_membership() {
    let app = create_test_router();
    let create_payload: CreateClassMembershipDto = payload_from_json(serde_json::json!({
        "class_id": Uuid::new_v4(),
        "user_id": TEST_USER_ID,
        "role": "student"
    }));

    let create_response =
        request_with_body(&app, Method::POST, "/class-memberships", &create_payload).await;
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_body: RestApiResponse<ClassMembershipDto> =
        deserialize_json_body(create_response.into_body())
            .await
            .unwrap();
    let created = create_body.0.data.unwrap();

    let update_payload: UpdateClassMembershipDto = payload_from_json(serde_json::json!({
        "id": created.id,
        "role": "ta"
    }));
    let update_response =
        request_with_body(&app, Method::PUT, "/class-memberships", &update_payload).await;
    assert_eq!(update_response.status(), StatusCode::OK);
    let update_body: RestApiResponse<ClassMembershipDto> =
        deserialize_json_body(update_response.into_body())
            .await
            .unwrap();
    let updated = update_body.0.data.unwrap();
    assert!(matches!(updated.role, ClassMembershipRole::Ta));

    let delete_url = format!("/class-memberships/{}", updated.id);
    let delete_response = request(&app, Method::DELETE, &delete_url).await;
    assert_eq!(delete_response.status(), StatusCode::OK);
}
