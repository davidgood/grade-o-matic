use std::{env, fs, path::PathBuf, sync::Once};

use axum::{
    Router,
    body::Body,
    http::{
        Method, Request, Response, StatusCode,
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    },
};

use dotenvy::from_filename;
use http_body_util::BodyExt;

use grade_o_matic_web::{
    app::create_router,
    common::{
        bootstrap::build_app_state,
        config::Config,
        dto::RestApiResponse,
        hash_util,
        jwt::{AuthBody, AuthPayload},
    },
};

use sqlx::{PgPool, postgres::PgPoolOptions};
use tower::ServiceExt;

static INIT: Once = Once::new();

/// Constants for test client credentials
/// These are used to authenticate the test client
#[allow(dead_code)]
pub const TEST_CLIENT_ID: &str = "apitest01";
#[allow(dead_code)]
pub const TEST_CLIENT_SECRET: &str = "test_password";

#[allow(dead_code)]
pub const TEST_USER_ID: uuid::Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000000");

/// Helper function to load environment variables from .env.test file
fn load_test_env() {
    INIT.call_once(|| {
        let _ = from_filename(".env.test");

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
        if env::var("DATABASE_URL").is_err() {
            unsafe {
                env::set_var(
                    "DATABASE_URL",
                    "postgres://postgres:postgres@127.0.0.1:5432/grade_o_matic_test",
                );
            }
        }

        // uncomment below for test debugging
        // use clean_axum_demo::common::bootstrap::setup_tracing;
        // setup_tracing();
    });
}

/// Helper function to set up the test database state
pub async fn setup_test_db() -> Result<PgPool, Box<dyn std::error::Error>> {
    let config = load_test_config()?;
    let database_url = config
        .database_url
        .as_deref()
        .ok_or("Missing DATABASE_URL (or DATABASE_* parts) for tests")?;

    let pool = PgPoolOptions::new()
        .max_connections(config.database_max_connections.unwrap_or(5))
        .min_connections(config.database_min_connections.unwrap_or(1))
        .connect(database_url)
        .await?;

    seed_test_auth_user(&pool).await?;

    Ok(pool)
}

async fn seed_test_auth_user(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let password_hash = hash_util::hash_password(TEST_CLIENT_SECRET)
        .map_err(|e| format!("failed to hash test password: {e}"))?;

    sqlx::query(
        r#"
        INSERT INTO users (id, username, email, created_by, modified_by)
        VALUES ($1, $2, $3, NULL, NULL)
        ON CONFLICT (id) DO UPDATE
        SET username = EXCLUDED.username, email = EXCLUDED.email, modified_at = NOW()
        "#,
    )
    .bind(TEST_USER_ID)
    .bind(TEST_CLIENT_ID)
    .bind("apitest01@example.test")
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO user_auth (user_id, password_hash)
        VALUES ($1, $2)
        ON CONFLICT (user_id) DO UPDATE
        SET password_hash = EXCLUDED.password_hash, modified_at = NOW()
        "#,
    )
    .bind(TEST_USER_ID)
    .bind(password_hash)
    .execute(pool)
    .await?;

    Ok(())
}

fn load_test_config() -> Result<Config, Box<dyn std::error::Error>> {
    load_test_env();

    match Config::from_env() {
        Ok(config) => Ok(config),
        Err(err)
            if err.to_string().contains(
                "OIDC is enabled, but one or more required OIDC_* variables are missing",
            ) =>
        {
            // Test fallback: disable OIDC if local env enables it without complete OIDC settings.
            unsafe {
                env::set_var("OIDC_ENABLED", "false");
            }
            Ok(Config::from_env()?)
        }
        Err(err) => Err(err.into()),
    }
}

/// Helper function to create a test router
pub async fn create_test_router() -> Router {
    let config = load_test_config().unwrap();
    ensure_test_assets(&config).expect("Failed to ensure test assets");
    let pool = setup_test_db().await.unwrap();
    let state = build_app_state(pool, config.clone());
    create_router(state)
}

fn ensure_test_assets(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/asset/cat.png")
        .canonicalize()?;
    let fixture = fixture_path.as_path();
    if !fixture.exists() {
        return Err("Missing tests/asset/cat.png fixture".into());
    }

    let public_target = PathBuf::from(&config.assets_public_path).join("images.jpeg");
    let private_target = PathBuf::from(&config.assets_private_path)
        .join("profile_picture")
        .join("images.jpeg");

    if let Some(parent) = public_target.parent() {
        fs::create_dir_all(parent)?;
    }
    if let Some(parent) = private_target.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::copy(fixture, &public_target)?;
    fs::copy(fixture, &private_target)?;

    Ok(())
}

/// Helper function gets the authentication token
/// for the test client
/// This function is used to authenticate the test client
#[allow(dead_code)]
async fn get_authentication_token() -> String {
    let payload = AuthPayload {
        client_id: TEST_CLIENT_ID.to_string(),
        client_secret: TEST_CLIENT_SECRET.to_string(),
    };

    let response = request_with_body(Method::POST, "/auth/login", &payload);

    let (parts, body) = response.await.into_parts();

    assert_eq!(parts.status, StatusCode::OK);

    let response_body: RestApiResponse<AuthBody> = deserialize_json_body(body).await.unwrap();
    let auth_body = response_body.0.data.unwrap();
    let token = format!("{} {}", auth_body.token_type, auth_body.access_token);
    token
}

/// Helper function to deserialize the body of a request into a specific type
pub async fn deserialize_json_body<T: serde::de::DeserializeOwned>(
    body: Body,
) -> Result<T, Box<dyn std::error::Error>> {
    let bytes = body
        .collect()
        .await
        .map_err(|e| {
            tracing::error!("Failed to collect response body: {}", e);
            e
        })?
        .to_bytes();

    if bytes.is_empty() {
        return Err(("Empty response body").into());
    }

    // Debugging output
    // Uncomment the following lines to print the response body
    // if let Ok(body) = std::str::from_utf8(&bytes) {
    //     println!("body = {body:?}");
    // }

    let parsed = serde_json::from_slice::<T>(&bytes)?;

    Ok(parsed)
}

/// Helper functions to create a request
#[allow(dead_code)]
pub async fn request(method: Method, uri: &str) -> Response<Body> {
    let request = get_request(method, uri);
    let app = create_test_router().await;

    app.oneshot(request.await).await.unwrap()
}

/// Helper function to create a request with a body
#[allow(dead_code)]
pub async fn request_with_body<T: serde::Serialize>(
    method: Method,
    uri: &str,
    payload: &T,
) -> Response<Body> {
    let json_payload = serde_json::to_string(payload).expect("Failed to serialize payload");
    let request = get_request_with_body(method, uri, &json_payload);
    let app = create_test_router().await;

    app.oneshot(request.await).await.unwrap()
}

/// Helper function to create a request with authentication
#[allow(dead_code)]
pub async fn request_with_auth(method: Method, uri: &str) -> Response<Body> {
    let token = get_authentication_token().await;
    let request = get_request_with_auth(method, uri, &token);
    let app = create_test_router().await;

    app.oneshot(request.await).await.unwrap()
}

/// Helper function to create a request with authentication and a body
#[allow(dead_code)]
pub async fn request_with_auth_and_body<T: serde::Serialize>(
    method: Method,
    uri: &str,
    payload: &T,
) -> Response<Body> {
    let json_payload = serde_json::to_string(payload).expect("Failed to serialize payload");
    let token = get_authentication_token().await;
    let request = get_request_with_auth_and_body(method, uri, &token, &json_payload);
    let app = create_test_router().await;

    app.oneshot(request.await).await.unwrap()
}

/// Helper function to create a request with authentication and multipart data
#[allow(dead_code)]
pub async fn request_with_auth_and_multipart(
    method: Method,
    uri: &str,
    payload: Vec<u8>,
) -> Response<Body> {
    let token = get_authentication_token().await;
    let request = get_request_with_auth_and_multipart(method, uri, &token, payload);
    let app = create_test_router().await;

    app.oneshot(request.await).await.unwrap()
}

/// internal helper functions to create requests
async fn get_request(method: Method, uri: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri.to_string())
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .body(axum::body::Body::empty())
        .unwrap()
}

/// internal helper function to create a request with a body
async fn get_request_with_body(method: Method, uri: &str, payload: &str) -> Request<Body> {
    let request: Request<Body> = Request::builder()
        .method(method)
        .uri(uri.to_string())
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .body(axum::body::Body::from(payload.to_string()))
        .unwrap();

    request
}

/// internal helper function to create a request with authorization
async fn get_request_with_auth(method: Method, uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri.to_string())
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, token)
        .header(ACCEPT, "application/json")
        .body(axum::body::Body::empty())
        .unwrap()
}

async fn get_request_with_auth_and_body(
    method: Method,
    uri: &str,
    token: &str,
    payload: &str,
) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri.to_string())
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, token)
        .header(ACCEPT, "application/json")
        .body(axum::body::Body::from(payload.to_string()))
        .unwrap()
}

async fn get_request_with_auth_and_multipart(
    method: Method,
    uri: &str,
    token: &str,
    payload: Vec<u8>,
) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri.to_string())
        .header(CONTENT_TYPE, "multipart/form-data; boundary=----XYZ")
        .header(AUTHORIZATION, token)
        .header(ACCEPT, "application/json")
        .body(Body::from(payload))
        .unwrap()
}
