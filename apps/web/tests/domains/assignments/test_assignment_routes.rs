use crate::test_helpers::{TEST_USER_ID, deserialize_json_body};
use async_trait::async_trait;
use axum::Router;
use axum::body::Body;
use axum::extract::FromRef;
use axum::http::{Method, Request, StatusCode};
use chrono::Utc;
use grade_o_matic_web::common::dto::RestApiResponse;
use grade_o_matic_web::common::error::AppError;
use grade_o_matic_web::common::jwt::Claims;
use grade_o_matic_web::domains::assignments::dto::assignment_dto::{
    AssignmentDto, AssignmentWithAttachmentCountDto, CreateAssignmentDto, UpdateAssignmentDto,
};
use grade_o_matic_web::domains::assignments::{
    AssignmentAttachment, AssignmentDeadlineType, AssignmentServiceTrait,
    StudentAssignmentSubmission, assignment_routes,
};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceExt;
use uuid::Uuid;

#[derive(Clone)]
struct TestState {
    assignment_service: Arc<dyn AssignmentServiceTrait>,
}
impl FromRef<TestState> for Arc<dyn AssignmentServiceTrait> {
    fn from_ref(state: &TestState) -> Self {
        Arc::clone(&state.assignment_service)
    }
}

#[derive(Default)]
struct AssignmentStore {
    assignments: HashMap<Uuid, AssignmentDto>,
}
struct FakeAssignmentService {
    store: Arc<Mutex<AssignmentStore>>,
}

impl FakeAssignmentService {
    fn new() -> Self {
        let seed_id = Uuid::new_v4();
        let seed = AssignmentDto {
            id: seed_id,
            class_id: Default::default(),
            title: "Seed Assignment".to_string(),
            description: Some("This is a seed assignment for testing purposes.".to_string()),
            due_at: Some(Utc::now()),
            deadline_type: AssignmentDeadlineType::SoftDeadline,
            points: Some(100),
        };

        let mut assignments = HashMap::new();
        assignments.insert(seed_id, seed);

        Self {
            store: Arc::new(Mutex::new(AssignmentStore { assignments })),
        }
    }
}

#[async_trait]
impl AssignmentServiceTrait for FakeAssignmentService {
    fn create_service(_pool: PgPool) -> Arc<dyn AssignmentServiceTrait>
    where
        Self: Sized,
    {
        Arc::new(Self::new())
    }

    async fn list(&self) -> Result<Vec<AssignmentDto>, AppError> {
        let store = self.store.lock().await;
        Ok(store.assignments.values().cloned().collect())
    }

    async fn list_by_class(&self, _class_id: Uuid) -> Result<Vec<AssignmentDto>, AppError> {
        let store = self.store.lock().await;
        Ok(store.assignments.values().cloned().collect())
    }

    async fn list_by_class_with_attachment_count(
        &self,
        _class_id: Uuid,
    ) -> Result<Vec<AssignmentWithAttachmentCountDto>, AppError> {
        let store = self.store.lock().await;
        Ok(store
            .assignments
            .values()
            .cloned()
            .map(|assignment| AssignmentWithAttachmentCountDto {
                id: assignment.id,
                class_id: assignment.class_id,
                title: assignment.title,
                description: assignment.description,
                due_at: assignment.due_at,
                deadline_type: assignment.deadline_type,
                points: assignment.points,
                attachment_count: 0,
            })
            .collect())
    }

    async fn list_attachments(
        &self,
        _assignment_id: Uuid,
    ) -> Result<Vec<AssignmentAttachment>, AppError> {
        Ok(vec![])
    }

    async fn list_student_submission_history(
        &self,
        _assignment_id: Uuid,
        _student_id: Uuid,
    ) -> Result<Vec<StudentAssignmentSubmission>, AppError> {
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

    async fn find_by_id(&self, id: Uuid) -> Result<Option<AssignmentDto>, AppError> {
        let store = self.store.lock().await;
        match store.assignments.get(&id).cloned() {
            Some(assignment) => Ok(Some(assignment)),
            None => Err(AppError::NotFound("Assignment not found".into())),
        }
    }

    async fn create(&self, assignment: CreateAssignmentDto) -> Result<AssignmentDto, AppError> {
        let id = Uuid::new_v4();

        let assignment = AssignmentDto {
            id,
            class_id: Default::default(),
            title: assignment.title,
            description: assignment.description,
            due_at: assignment.due_at,
            deadline_type: assignment.deadline_type,
            points: None,
        };

        let mut store = self.store.lock().await;
        store.assignments.insert(id, assignment.clone());
        Ok(assignment)
    }

    async fn update(&self, payload: UpdateAssignmentDto) -> Result<AssignmentDto, AppError> {
        let mut store = self.store.lock().await;
        let assignment = store
            .assignments
            .get_mut(&payload.id)
            .ok_or_else(|| AppError::NotFound("Assignment not found".into()))?;

        assignment.title = payload.title.clone();
        assignment.description = payload.description.clone();
        assignment.due_at = payload.due_at;
        assignment.deadline_type = payload.deadline_type;

        Ok(assignment.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<String, AppError> {
        let mut store = self.store.lock().await;
        if store.assignments.remove(&id).is_some() {
            Ok("Assignment deleted".to_string())
        } else {
            Err(AppError::NotFound("Assignment not found".into()))
        }
    }
}

fn create_test_router() -> Router {
    let state = TestState {
        assignment_service: Arc::new(FakeAssignmentService::new()),
    };

    Router::new()
        .nest("/assignments", assignment_routes::<TestState>())
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

async fn create_assignment(app: &Router) -> (CreateAssignmentDto, AssignmentDto) {
    let class_id = Uuid::new_v4();

    let payload = CreateAssignmentDto {
        class_id,
        title: "Test Assignment".to_string(),
        description: None,
        due_at: Some(Utc::now()),
        deadline_type: AssignmentDeadlineType::SoftDeadline,
        points: Some(100),
        modified_by: TEST_USER_ID,
    };

    let response = request_with_auth_and_body(app, Method::POST, "/assignments", &payload).await;

    assert_eq!(response.status(), StatusCode::CREATED);

    let response_body: RestApiResponse<AssignmentDto> =
        deserialize_json_body(response.into_body()).await.unwrap();

    assert_eq!(response_body.0.status, StatusCode::CREATED.as_u16());

    let assignment_dto = response_body.0.data.unwrap();
    (payload, assignment_dto)
}

#[tokio::test]
async fn test_get_assignments() {
    let app = create_test_router();
    let response = request_with_auth(&app, Method::GET, "/assignments").await;
    let (parts, body) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<Vec<AssignmentDto>> =
        deserialize_json_body(body).await.unwrap();
    let assignments = response_body.0.data.unwrap();

    assert!(!assignments.is_empty());
}

#[tokio::test]
async fn test_get_assignment_by_id() {
    let app = create_test_router();
    let (_, assignment) = create_assignment(&app).await;

    let url = format!("/assignments/{}", assignment.id);

    let response = request_with_auth(&app, Method::GET, &url).await;
    assert_eq!(response.status(), StatusCode::OK);

    let response_body: RestApiResponse<AssignmentDto> =
        deserialize_json_body(response.into_body()).await.unwrap();
    let assignment_dto = response_body.0.data.unwrap();

    assert_eq!(assignment.id, assignment_dto.id);
    assert_eq!(assignment.title, assignment_dto.title);
    assert_eq!(assignment.description, assignment_dto.description);
    assert_eq!(assignment.due_at, assignment_dto.due_at);
}

#[tokio::test]
async fn test_create_assignment() {
    let app = create_test_router();
    let (payload, assignment_dto) = create_assignment(&app).await;

    assert_eq!(assignment_dto.title, payload.title);
    assert_eq!(assignment_dto.description, payload.description);
    assert_eq!(assignment_dto.due_at, payload.due_at);
    assert_eq!(assignment_dto.deadline_type, payload.deadline_type);
}

#[tokio::test]
async fn test_update_assignment() {
    let app = create_test_router();
    let (_, assignment) = create_assignment(&app).await;

    let payload = UpdateAssignmentDto {
        id: assignment.id,
        class_id: Default::default(),
        title: "Updated Title".to_string(),
        description: Some("Updated Description".to_string()),
        due_at: Some(Utc::now()),
        deadline_type: AssignmentDeadlineType::HardCutoff,
        points: Some(100),
        modified_by: Default::default(),
    };

    let url = format!("/assignments/{}", assignment.id);
    let response = request_with_auth_and_body(&app, Method::PUT, &url, &payload).await;
    assert_eq!(response.status(), StatusCode::OK);

    let response_body: RestApiResponse<AssignmentDto> =
        deserialize_json_body(response.into_body()).await.unwrap();
    let assignment_dto = response_body.0.data.unwrap();

    assert_eq!(assignment_dto.title, payload.title);
    assert_eq!(assignment_dto.description, payload.description);
    assert_eq!(assignment_dto.due_at, payload.due_at);
    assert_eq!(assignment_dto.deadline_type, payload.deadline_type);
}

#[tokio::test]
async fn test_delete_assignment() {
    let app = create_test_router();
    let url = format!("/assignments/{}", Uuid::new_v4());
    let response = request_with_auth(&app, Method::DELETE, &url).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let response_body: RestApiResponse<String> =
        deserialize_json_body(response.into_body()).await.unwrap();

    assert_eq!(response_body.0.status, StatusCode::NOT_FOUND.as_u16());
}
