use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use axum::{
    Router,
    body::Body,
    extract::FromRef,
    http::{Method, Request, StatusCode},
};
use chrono::{Duration, Utc};
use grade_o_matic::{
    common::{dto::RestApiResponse, error::AppError, jwt::Claims},
    domains::device::{
        DeviceOS, DeviceServiceTrait, DeviceStatus, device_routes,
        dto::device_dto::{
            CreateDeviceDto, DeviceDto, UpdateDeviceDto, UpdateDeviceDtoWithIdDto,
            UpdateManyDevicesDto,
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
    device_service: Arc<dyn DeviceServiceTrait>,
}

impl FromRef<TestState> for Arc<dyn DeviceServiceTrait> {
    fn from_ref(state: &TestState) -> Self {
        Arc::clone(&state.device_service)
    }
}

#[derive(Default)]
struct DeviceStore {
    devices: HashMap<Uuid, DeviceDto>,
}

struct FakeDeviceService {
    store: Arc<Mutex<DeviceStore>>,
}

impl FakeDeviceService {
    fn new() -> Self {
        let seed_id = Uuid::new_v4();
        let seed = DeviceDto {
            id: seed_id,
            user_id: TEST_USER_ID,
            name: "seed-device".to_string(),
            device_os: DeviceOS::Android,
            status: DeviceStatus::Active,
            registered_at: Some(Utc::now()),
            created_by: Some(TEST_USER_ID),
            created_at: Some(Utc::now()),
            modified_by: Some(TEST_USER_ID),
            modified_at: Some(Utc::now()),
        };

        let mut devices = HashMap::new();
        devices.insert(seed_id, seed);

        Self {
            store: Arc::new(Mutex::new(DeviceStore { devices })),
        }
    }
}

#[async_trait]
impl DeviceServiceTrait for FakeDeviceService {
    fn create_service(_pool: sqlx::PgPool) -> Arc<dyn DeviceServiceTrait>
    where
        Self: Sized,
    {
        Arc::new(Self::new())
    }

    async fn get_device_by_id(&self, id: Uuid) -> Result<DeviceDto, AppError> {
        let store = self.store.lock().await;
        store
            .devices
            .get(&id)
            .cloned()
            .ok_or_else(|| AppError::NotFound("Device not found".into()))
    }

    async fn get_devices(&self) -> Result<Vec<DeviceDto>, AppError> {
        let store = self.store.lock().await;
        Ok(store.devices.values().cloned().collect())
    }

    async fn create_device(&self, payload: CreateDeviceDto) -> Result<DeviceDto, AppError> {
        let now = Utc::now();
        let dto = DeviceDto {
            id: Uuid::new_v4(),
            user_id: payload.user_id,
            name: payload.name,
            device_os: payload.device_os,
            status: payload.status,
            registered_at: payload.registered_at,
            created_by: Some(payload.modified_by),
            created_at: Some(now),
            modified_by: Some(payload.modified_by),
            modified_at: Some(now),
        };

        let mut store = self.store.lock().await;
        store.devices.insert(dto.id, dto.clone());
        Ok(dto)
    }

    async fn update_device(
        &self,
        id: Uuid,
        payload: UpdateDeviceDto,
    ) -> Result<DeviceDto, AppError> {
        let mut store = self.store.lock().await;
        let device = store
            .devices
            .get_mut(&id)
            .ok_or_else(|| AppError::NotFound("Device not found".into()))?;

        if let Some(name) = payload.name {
            device.name = name;
        }
        if let Some(user_id) = payload.user_id {
            device.user_id = user_id;
        }
        if let Some(device_os) = payload.device_os {
            device.device_os = device_os;
        }
        if let Some(status) = payload.status {
            device.status = status;
        }
        if let Some(registered_at) = payload.registered_at {
            device.registered_at = Some(registered_at);
        }
        device.modified_by = Some(payload.modified_by);
        device.modified_at = Some(Utc::now());

        Ok(device.clone())
    }

    async fn delete_device(&self, id: Uuid) -> Result<String, AppError> {
        let mut store = self.store.lock().await;
        if store.devices.remove(&id).is_some() {
            Ok("Device deleted".to_string())
        } else {
            Err(AppError::NotFound("Device not found".into()))
        }
    }

    async fn update_many_devices(
        &self,
        user_id: Uuid,
        modified_by: Uuid,
        payload: UpdateManyDevicesDto,
    ) -> Result<String, AppError> {
        let mut store = self.store.lock().await;
        let now = Utc::now();

        for item in payload.devices {
            match item.id {
                Some(id) => {
                    if let Some(device) = store.devices.get_mut(&id) {
                        device.name = item.name;
                        device.device_os = item.device_os;
                        device.status = item.status;
                        device.modified_by = Some(modified_by);
                        device.modified_at = Some(now);
                    }
                }
                None => {
                    let id = Uuid::new_v4();
                    store.devices.insert(
                        id,
                        DeviceDto {
                            id,
                            user_id,
                            name: item.name,
                            device_os: item.device_os,
                            status: item.status,
                            registered_at: Some(now),
                            created_by: Some(modified_by),
                            created_at: Some(now),
                            modified_by: Some(modified_by),
                            modified_at: Some(now),
                        },
                    );
                }
            }
        }

        Ok("Devices updated".to_string())
    }
}

fn create_test_router() -> Router {
    let state = TestState {
        device_service: Arc::new(FakeDeviceService::new()),
    };

    Router::new()
        .nest("/device", device_routes::<TestState>())
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

async fn create_test_device(app: &Router) -> DeviceDto {
    let name = format!("test-device-{}", Uuid::new_v4());

    let payload = CreateDeviceDto {
        name,
        user_id: TEST_USER_ID,
        device_os: DeviceOS::Android,
        status: DeviceStatus::Active,
        registered_at: Some(Utc::now() + Duration::minutes(30)),
        modified_by: TEST_USER_ID,
    };

    let response = request_with_auth_and_body(app, Method::POST, "/device", &payload).await;
    let (parts, body) = response.into_parts();
    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<DeviceDto> = deserialize_json_body(body).await.unwrap();
    response_body.0.data.unwrap()
}

#[tokio::test]
async fn test_create_device() {
    let app = create_test_router();
    let name = format!("test-device-{}", Uuid::new_v4());

    let payload = CreateDeviceDto {
        name,
        user_id: TEST_USER_ID,
        device_os: DeviceOS::Android,
        status: DeviceStatus::Active,
        registered_at: Some(Utc::now() + Duration::minutes(30)),
        modified_by: TEST_USER_ID,
    };

    let response = request_with_auth_and_body(&app, Method::POST, "/device", &payload).await;
    let (parts, body) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<DeviceDto> = deserialize_json_body(body).await.unwrap();
    let device_dto = response_body.0.data.unwrap();

    assert_eq!(device_dto.name, payload.name);
    assert_eq!(device_dto.user_id, payload.user_id);
    assert_eq!(device_dto.device_os, payload.device_os);
    assert_eq!(device_dto.status, payload.status);
    assert_eq!(
        device_dto.registered_at.map(|dt| dt.timestamp()),
        payload.registered_at.map(|dt| dt.timestamp())
    );
    assert_eq!(device_dto.modified_by, Some(TEST_USER_ID));
}

#[tokio::test]
async fn test_get_devices() {
    let app = create_test_router();
    let response = request_with_auth(&app, Method::GET, "/device").await;
    let (parts, body) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<Vec<DeviceDto>> = deserialize_json_body(body).await.unwrap();
    let devices = response_body.0.data.unwrap();

    assert!(!devices.is_empty());
}

#[tokio::test]
async fn test_get_device_by_id() {
    let app = create_test_router();
    let device = create_test_device(&app).await;
    let url = format!("/device/{}", device.id);

    let response = request_with_auth(&app, Method::GET, &url).await;
    let (parts, body) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<DeviceDto> = deserialize_json_body(body).await.unwrap();
    let response_device = response_body.0.data.unwrap();

    assert_eq!(response_device.id, device.id);
    assert_eq!(response_device.name, device.name);
    assert_eq!(response_device.user_id, device.user_id);
    assert_eq!(response_device.device_os, device.device_os);
    assert_eq!(response_device.status, device.status);
}

#[tokio::test]
async fn test_update_device() {
    let app = create_test_router();
    let existent_device = create_test_device(&app).await;

    let payload = UpdateDeviceDto {
        name: Some(format!("update-device-{}", Uuid::new_v4())),
        user_id: Some(existent_device.user_id),
        device_os: Some(DeviceOS::IOS),
        status: Some(DeviceStatus::Decommissioned),
        registered_at: Some(Utc::now() + Duration::minutes(30)),
        modified_by: TEST_USER_ID,
    };

    let url = format!("/device/{}", existent_device.id);
    let response = request_with_auth_and_body(&app, Method::PUT, &url, &payload).await;
    let (parts, body) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<DeviceDto> = deserialize_json_body(body).await.unwrap();
    let response_device = response_body.0.data.unwrap();

    assert_eq!(response_device.id, existent_device.id);
    assert_eq!(Some(response_device.name), payload.name);
    assert_eq!(Some(response_device.user_id), payload.user_id);
    assert_eq!(Some(response_device.device_os), payload.device_os);
    assert_eq!(Some(response_device.status), payload.status);
    assert_eq!(
        response_device.registered_at.map(|dt| dt.timestamp()),
        payload.registered_at.map(|dt| dt.timestamp())
    );
}

#[tokio::test]
async fn test_delete_device_not_found() {
    let app = create_test_router();
    let non_existent_id = Uuid::new_v4();
    let url = format!("/device/{}", non_existent_id);

    let response = request_with_auth(&app, Method::DELETE, &url).await;
    let (parts, body) = response.into_parts();

    assert_eq!(parts.status, StatusCode::NOT_FOUND);

    let response_body: RestApiResponse<()> = deserialize_json_body(body).await.unwrap();
    assert_eq!(response_body.0.status, StatusCode::NOT_FOUND.as_u16());
}

#[tokio::test]
async fn test_delete_device() {
    let app = create_test_router();
    let existent_device = create_test_device(&app).await;
    let url = format!("/device/{}", existent_device.id);

    let response = request_with_auth(&app, Method::DELETE, &url).await;
    let (parts, body) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<()> = deserialize_json_body(body).await.unwrap();
    assert_eq!(response_body.0.status, StatusCode::OK.as_u16());
}

#[tokio::test]
async fn test_update_many_devices() {
    let app = create_test_router();
    let existent_device = create_test_device(&app).await;

    let payload = UpdateManyDevicesDto {
        devices: vec![
            UpdateDeviceDtoWithIdDto {
                id: Some(existent_device.id),
                name: format!("many-update-device-{}", Uuid::new_v4()),
                device_os: DeviceOS::IOS,
                status: DeviceStatus::Blocked,
            },
            UpdateDeviceDtoWithIdDto {
                id: None,
                name: format!("many-update-in-device-{}", Uuid::new_v4()),
                device_os: DeviceOS::Android,
                status: DeviceStatus::Pending,
            },
        ],
    };

    let url = format!("/device/batch/{}", TEST_USER_ID);
    let response = request_with_auth_and_body(&app, Method::PUT, &url, &payload).await;
    let (parts, body) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<()> = deserialize_json_body(body).await.unwrap();
    assert_eq!(response_body.0.status, StatusCode::OK.as_u16());
}
