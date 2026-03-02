use std::sync::Arc;

use sqlx::PgPool;

use crate::common::config::Config;
use crate::domains::assignments::{AssignmentServiceTrait, create_assignment_service};
use crate::domains::auth::{AuthService, AuthServiceTrait};
use crate::domains::class_memberships::{
    ClassMembershipServiceTrait, create_class_membership_service,
};
use crate::domains::device::{DeviceService, DeviceServiceTrait};
use crate::domains::file::{FileService, FileServiceTrait};
use crate::domains::user::UserServiceTrait;
use crate::{
    common::app_state::{AppServices, AppState},
    domains::user::UserService,
};

use crate::domains::classes::{ClassServiceTrait, create_class_service};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Constructs and wires all application services and returns a configured AppState.
pub fn build_app_state(pool: PgPool, config: Config) -> AppState {
    let auth_service: Arc<dyn AuthServiceTrait> = AuthService::create_service(pool.clone());
    let file_service: Arc<dyn FileServiceTrait> =
        FileService::create_service(config.clone(), pool.clone());
    let user_service: Arc<dyn UserServiceTrait> =
        UserService::create_service(pool.clone(), Arc::clone(&file_service));
    let assignment_service: Arc<dyn AssignmentServiceTrait> =
        create_assignment_service(pool.clone());
    let device_service: Arc<dyn DeviceServiceTrait> = DeviceService::create_service(pool.clone());
    let class_service: Arc<dyn ClassServiceTrait> = create_class_service(pool.clone());
    let class_membership_service: Arc<dyn ClassMembershipServiceTrait> =
        create_class_membership_service(pool.clone());

    AppState::new(
        config,
        AppServices {
            auth_service,
            user_service,
            assignment_service,
            device_service,
            file_service,
            class_service,
            class_membership_service,
        },
    )
}

/// Setup tracing for the application.
/// This function initializes the tracing subscriber with a default filter and formatting.
pub fn setup_tracing() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=info,tower_http=info,axum::rejection=trace".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_file(true)
                .with_line_number(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_target(true)
                .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE),
        )
        .init();
}

/// Shutdown signal handler
/// This function listens for a shutdown signal (CTRL+C) and logs a message when received.
pub async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}
