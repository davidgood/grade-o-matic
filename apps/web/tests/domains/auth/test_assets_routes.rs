use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode, header::AUTHORIZATION},
    middleware,
    routing::get,
};
use grade_o_matic_web::common::{config::Config, jwt};
use grade_o_matic_web::domains::user::UserRole;
use tower::ServiceExt;
use tower_http::services::ServeDir;

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use crate::test_helpers::TEST_USER_ID;

fn load_assets_test_config() -> Config {
    if env::var("JWT_SECRET_KEY").is_err() {
        unsafe {
            env::set_var("JWT_SECRET_KEY", "ci-test-jwt-secret");
        }
    }
    if env::var("OIDC_ENABLED").is_err() {
        unsafe {
            env::set_var("OIDC_ENABLED", "false");
        }
    }

    match Config::from_env() {
        Ok(config) => config,
        Err(err)
            if err.to_string().contains(
                "OIDC is enabled, but one or more required OIDC_* variables are missing",
            ) =>
        {
            // Keep asset tests independent from unrelated auth provider config.
            unsafe {
                env::set_var("OIDC_ENABLED", "false");
            }
            Config::from_env().expect("config should load after disabling OIDC in tests")
        }
        Err(err) => panic!("failed to load test config: {err}"),
    }
}

fn ensure_test_assets(config: &Config) {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/asset/cat.png")
        .canonicalize()
        .expect("fixture path should resolve");
    let fixture = fixture_path.as_path();
    assert!(fixture.exists(), "missing fixture tests/asset/cat.png");

    let public_target = Path::new(&config.assets_public_path).join("images.jpeg");
    let private_target = Path::new(&config.assets_private_path)
        .join("profile_picture")
        .join("images.jpeg");

    fs::create_dir_all(public_target.parent().expect("public parent")).unwrap();
    fs::create_dir_all(private_target.parent().expect("private parent")).unwrap();
    fs::copy(fixture, public_target).unwrap();
    fs::copy(fixture, private_target).unwrap();
}

fn create_assets_test_router(config: &Config) -> Router {
    let public_assets_routes = Router::new().nest_service(
        config.assets_public_url.as_str(),
        ServeDir::new(config.assets_public_path.clone()),
    );

    let private_assets_routes = Router::new()
        .nest_service(
            config.assets_private_url.as_str(),
            ServeDir::new(config.assets_private_path.clone()),
        )
        .route_layer(middleware::from_fn(jwt::jwt_auth));

    Router::new()
        .merge(public_assets_routes)
        .merge(private_assets_routes)
        .route("/health", get(|| async { "ok" }))
}

async fn request(router: &Router, method: Method, uri: &str) -> axum::response::Response {
    let req = Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    router.clone().oneshot(req).await.unwrap()
}

async fn request_with_auth(router: &Router, method: Method, uri: &str) -> axum::response::Response {
    let token = jwt::make_jwt_token(&TEST_USER_ID, UserRole::Student).unwrap();
    let req = Request::builder()
        .method(method)
        .uri(uri)
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    router.clone().oneshot(req).await.unwrap()
}

#[tokio::test]
async fn test_public_assets() {
    let config = load_assets_test_config();
    ensure_test_assets(&config);
    let app = create_assets_test_router(&config);
    let public_uri = format!("{}/images.jpeg", config.assets_public_url);
    let response = request(&app, Method::GET, &public_uri);

    let (parts, _) = response.await.into_parts();

    assert_eq!(parts.status, StatusCode::OK);
}

#[tokio::test]
async fn test_private_assets_without_auth() {
    let config = load_assets_test_config();
    ensure_test_assets(&config);
    let app = create_assets_test_router(&config);
    let private_uri = format!("{}/profile_picture/images.jpeg", config.assets_private_url);
    let response = request(&app, Method::GET, &private_uri);

    let (parts, _) = response.await.into_parts();
    assert_eq!(parts.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_private_assets_with_auth() {
    let config = load_assets_test_config();
    ensure_test_assets(&config);
    let app = create_assets_test_router(&config);
    let private_uri = format!("{}/profile_picture/images.jpeg", config.assets_private_url);
    let response = request_with_auth(&app, Method::GET, &private_uri);

    let (parts, _) = response.await.into_parts();
    assert_eq!(parts.status, StatusCode::OK);
}
