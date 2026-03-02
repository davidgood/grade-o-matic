use axum::response::Html;
use axum::{
    extract::{Form, State},
    http::{StatusCode, header::SET_COOKIE},
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use minijinja::context;
use std::sync::Arc;

use crate::common::error::AppError;
use crate::common::jwt::AuthPayload;
use crate::domains::auth::AuthServiceTrait;

use super::render_template;

pub async fn ui_index() -> Result<Html<String>, AppError> {
    let html = render_template("index.html", context! { title => "Grade-O-Matic" })?;
    Ok(Html(html))
}

pub async fn server_time_fragment() -> Html<String> {
    let now = Utc::now().to_rfc3339();
    Html(format!("<code>Server time: {now}</code>"))
}

#[derive(serde::Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

pub async fn login_page() -> Result<Html<String>, AppError> {
    let html = render_template(
        "login.html",
        context! {
            title => "Login",
            error => "",
            username => "",
        },
    )?;
    Ok(Html(html))
}

pub async fn login_submit(
    State(auth_service): State<Arc<dyn AuthServiceTrait>>,
    Form(form): Form<LoginForm>,
) -> Result<Response, AppError> {
    let payload = AuthPayload {
        client_id: form.username.clone(),
        client_secret: form.password,
    };

    match auth_service.login_user(payload).await {
        Ok(auth_body) => {
            let cookie = format!(
                "auth_token={}; Path=/; HttpOnly; SameSite=Lax",
                auth_body.access_token
            );
            Ok((
                StatusCode::SEE_OTHER,
                [(SET_COOKIE, cookie)],
                Redirect::to("/"),
            )
                .into_response())
        }
        Err(err) => {
            let html = render_template(
                "login.html",
                context! {
                    title => "Login",
                    error => err.to_string(),
                    username => form.username,
                },
            )?;
            Ok((StatusCode::UNAUTHORIZED, Html(html)).into_response())
        }
    }
}

pub async fn logout() -> impl IntoResponse {
    (
        StatusCode::SEE_OTHER,
        [(
            SET_COOKIE,
            "auth_token=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax".to_string(),
        )],
        Redirect::to("/ui/login"),
    )
}
