use axum::{
    body::Body,
    http::{header::AUTHORIZATION, Method, Request, StatusCode},
    middleware,
    routing::get,
    Router,
};
use grade_o_matic::common::{config::Config, jwt};
use tower::ServiceExt;
use tower_http::services::ServeDir;

use std::{env, fs, path::Path};

mod test_helpers;

use test_helpers::TEST_USER_ID;

fn load_assets_test_config() -> Config {
    dotenvy::from_filename(".env.test")
        .or_else(|_| dotenvy::dotenv())
        .ok();

    match Config::from_env() {
        Ok(config) => config,
        Err(err)
            if err
                .to_string()
                .contains("OIDC is enabled, but one or more required OIDC_* variables are missing") =>
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
    let fixture = Path::new("tests/asset/cat.png");
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
    let token = jwt::make_jwt_token(TEST_USER_ID).unwrap();
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
    let response = request(&app, Method::GET, "/public/images.jpeg");

    let (parts, _) = response.await.into_parts();

    assert_eq!(parts.status, StatusCode::OK);
}

#[tokio::test]
async fn test_private_assets_without_auth() {
    let config = load_assets_test_config();
    ensure_test_assets(&config);
    let app = create_assets_test_router(&config);
    let response = request(&app, Method::GET, "/private/profile_picture/images.jpeg");

    let (parts, _) = response.await.into_parts();
    assert_eq!(parts.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_private_assets_with_auth() {
    let config = load_assets_test_config();
    ensure_test_assets(&config);
    let app = create_assets_test_router(&config);
    let response = request_with_auth(&app, Method::GET, "/private/profile_picture/images.jpeg");

    let (parts, _) = response.await.into_parts();
    assert_eq!(parts.status, StatusCode::OK);
}
