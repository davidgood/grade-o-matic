use std::sync::Arc;

use axum::extract::FromRef;

use super::config::Config;
use crate::domains::classes::ClassServiceTrait;
use crate::domains::{
    assignments::AssignmentServiceTrait,
    auth::AuthServiceTrait,
    class_memberships::ClassMembershipServiceTrait,
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
    /// Service handling class membership-related logic.
    pub class_membership_service: Arc<dyn ClassMembershipServiceTrait>,
}

/// Groups all service dependencies for building [`AppState`].
#[derive(Clone)]
pub struct AppServices {
    pub auth_service: Arc<dyn AuthServiceTrait>,
    pub user_service: Arc<dyn UserServiceTrait>,
    pub assignment_service: Arc<dyn AssignmentServiceTrait>,
    pub device_service: Arc<dyn DeviceServiceTrait>,
    pub file_service: Arc<dyn FileServiceTrait>,
    pub class_service: Arc<dyn ClassServiceTrait>,
    pub class_membership_service: Arc<dyn ClassMembershipServiceTrait>,
}

impl AppState {
    /// Creates a new instance of AppState with the provided dependencies.
    pub fn new(config: Config, services: AppServices) -> Self {
        Self {
            config,
            auth_service: services.auth_service,
            user_service: services.user_service,
            assignment_service: services.assignment_service,
            device_service: services.device_service,
            file_service: services.file_service,
            class_service: services.class_service,
            class_membership_service: services.class_membership_service,
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

impl FromRef<AppState> for Arc<dyn ClassMembershipServiceTrait> {
    fn from_ref(input: &AppState) -> Self {
        Arc::clone(&input.class_membership_service)
    }
}
