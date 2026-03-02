use std::sync::Arc;

use axum::extract::FromRef;

use super::config::Config;
use crate::domains::classes::ClassServiceTrait;
use crate::domains::{
    assignments::AssignmentServiceTrait,
    auth::AuthServiceTrait,
    device::DeviceServiceTrait,
    file::FileServiceTrait,
    user::{UserAssetPattern, UserServiceTrait},
};

/// AppState is a struct that holds the application-wide shared state.
/// It is passed to request handlers via Axum's extension mechanism.
#[derive(Clone)]
pub struct AppState {
    /// Global application configuration.
    pub config: Config,
    /// Service handling authentication-related logic.
    pub auth_service: Arc<dyn AuthServiceTrait>,
    /// Service handling user-related logic.
    pub user_service: Arc<dyn UserServiceTrait>,
    /// Service handling assignment-related logic.
    pub assignment_service: Arc<dyn AssignmentServiceTrait>,
    /// Service handling device-related logic.
    pub device_service: Arc<dyn DeviceServiceTrait>,
    /// Service handling file-related logic.
    pub file_service: Arc<dyn FileServiceTrait>,
    /// Service handling class-related logic.
    pub class_service: Arc<dyn ClassServiceTrait>,
}

impl AppState {
    /// Creates a new instance of AppState with the provided dependencies.
    pub fn new(
        config: Config,
        auth_service: Arc<dyn AuthServiceTrait>,
        user_service: Arc<dyn UserServiceTrait>,
        assignment_service: Arc<dyn AssignmentServiceTrait>,
        device_service: Arc<dyn DeviceServiceTrait>,
        file_service: Arc<dyn FileServiceTrait>,
        class_service: Arc<dyn ClassServiceTrait>,
    ) -> Self {
        Self {
            config,
            auth_service,
            user_service,
            assignment_service,
            device_service,
            file_service,
            class_service,
        }
    }
}

impl FromRef<AppState> for Arc<dyn AuthServiceTrait> {
    fn from_ref(input: &AppState) -> Self {
        Arc::clone(&input.auth_service)
    }
}

impl FromRef<AppState> for Arc<dyn DeviceServiceTrait> {
    fn from_ref(input: &AppState) -> Self {
        Arc::clone(&input.device_service)
    }
}

impl FromRef<AppState> for Arc<dyn UserServiceTrait> {
    fn from_ref(input: &AppState) -> Self {
        Arc::clone(&input.user_service)
    }
}

impl FromRef<AppState> for Arc<dyn AssignmentServiceTrait> {
    fn from_ref(input: &AppState) -> Self {
        Arc::clone(&input.assignment_service)
    }
}

impl FromRef<AppState> for UserAssetPattern {
    fn from_ref(input: &AppState) -> Self {
        UserAssetPattern(input.config.asset_allowed_extensions_pattern.clone())
    }
}

impl FromRef<AppState> for Arc<dyn ClassServiceTrait> {
    fn from_ref(input: &AppState) -> Self {
        Arc::clone(&input.class_service)
    }
}
