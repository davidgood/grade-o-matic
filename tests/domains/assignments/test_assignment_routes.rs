use crate::test_helpers::TEST_USER_ID;
use axum::{
    Router,
    body::Body,
    http::{Method, Request, header::AUTHORIZATION},
};
use grade_o_matic::common::jwt;
use tower::ServiceExt;

async fn request(router: &Router, method: Method, uri: &str) -> axum::response::Response {
    let req = Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    router.clone().oneshot(req).await.unwrap()
}

async fn request_with_auth(router: &Router, method: Method, uri: &str) -> axum::response::Response {
    let token = jwt::make_jwt_token(&TEST_USER_ID).unwrap();
    let req = Request::builder()
        .method(method)
        .uri(uri)
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    router.clone().oneshot(req).await.unwrap()
}
