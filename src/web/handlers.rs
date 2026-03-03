use axum::{Extension, response::Html};
use axum::{
    extract::{Form, State},
    http::{StatusCode, header::SET_COOKIE},
    response::{IntoResponse, Redirect, Response},
};
use axum_csrf::CsrfToken;
use chrono::Utc;
use minijinja::context;
use std::sync::Arc;

use crate::common::error::AppError;
use crate::common::extractors::AdminUser;
use crate::common::jwt::AuthPayload;
use crate::domains::auth::AuthServiceTrait;
use crate::domains::auth::dto::auth_dto::AuthUserDto;
use crate::domains::user::dto::user_dto::CreateUserMultipartDto;
use crate::domains::user::{UserRole, UserServiceTrait};

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
    authenticity_token: String,
}

pub async fn login_page(token: CsrfToken) -> Result<Response, AppError> {
    let authenticity_token = token
        .authenticity_token()
        .map_err(|_| AppError::InternalError)?;

    let html = render_template(
        "login.html",
        context! {
            title => "Login",
            error => "",
            username => "",
            authenticity_token => authenticity_token,
        },
    )?;
    Ok((token, Html(html)).into_response())
}

pub async fn login_submit(
    State(auth_service): State<Arc<dyn AuthServiceTrait>>,
    token: CsrfToken,
    Form(form): Form<LoginForm>,
) -> Result<Response, AppError> {
    if token.verify(&form.authenticity_token).is_err() {
        return Err(AppError::Forbidden);
    }

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
                    authenticity_token => token.authenticity_token().unwrap_or_default(),
                },
            )?;
            let mut response = (token, Html(html)).into_response();
            *response.status_mut() = StatusCode::UNAUTHORIZED;
            Ok(response)
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

#[derive(serde::Deserialize)]
pub struct AdminCreateUserForm {
    username: String,
    email: String,
    role: String,
    password: String,
    authenticity_token: String,
}

pub async fn admin_users_page(_admin: AdminUser, token: CsrfToken) -> Result<Response, AppError> {
    let authenticity_token = token
        .authenticity_token()
        .map_err(|_| AppError::InternalError)?;

    let html = render_template(
        "admin/users_new.html",
        context! {
            title => "Create User Account",
            error => "",
            success => "",
            username => "",
            email => "",
            role => "student",
            authenticity_token => authenticity_token,
        },
    )?;
    Ok((token, Html(html)).into_response())
}

pub async fn admin_create_user_submit(
    State(user_service): State<Arc<dyn UserServiceTrait>>,
    State(auth_service): State<Arc<dyn AuthServiceTrait>>,
    Extension(claims): Extension<crate::common::jwt::Claims>,
    token: CsrfToken,
    Form(form): Form<AdminCreateUserForm>,
) -> Result<Response, AppError> {
    if token.verify(&form.authenticity_token).is_err() {
        return Err(AppError::Forbidden);
    }

    let user_role = parse_user_role(&form.role)?;

    let create_user = CreateUserMultipartDto {
        username: form.username.clone(),
        email: form.email.clone(),
        modified_by: claims.sub,
        profile_picture: None,
        user_role: user_role.clone(),
    };

    let created_user = match user_service.create_user(create_user, None).await {
        Ok(user) => user,
        Err(err) => {
            let html = render_template(
                "admin/users_new.html",
                context! {
                    title => "Create User Account",
                    error => err.to_string(),
                    success => "",
                    username => form.username,
                    email => form.email,
                    role => form.role,
                    authenticity_token => token.authenticity_token().unwrap_or_default(),
                },
            )?;
            let mut response = (token, Html(html)).into_response();
            *response.status_mut() = StatusCode::BAD_REQUEST;
            return Ok(response);
        }
    };

    let auth_payload = AuthUserDto {
        user_id: created_user.id,
        password: form.password,
    };

    if let Err(err) = auth_service.create_user_auth(auth_payload).await {
        let _ = user_service.delete_user(created_user.id).await;
        let html = render_template(
            "admin/users_new.html",
            context! {
                title => "Create User Account",
                error => err.to_string(),
                success => "",
                username => form.username,
                email => form.email,
                role => form.role,
                authenticity_token => token.authenticity_token().unwrap_or_default(),
            },
        )?;
        let mut response = (token, Html(html)).into_response();
        *response.status_mut() = StatusCode::BAD_REQUEST;
        return Ok(response);
    }

    let html = render_template(
        "admin/users_new.html",
        context! {
            title => "Create User Account",
            error => "",
            success => "User account created successfully.",
            username => "",
            email => "",
            role => "student",
            authenticity_token => token.authenticity_token().unwrap_or_default(),
        },
    )?;
    let mut response = (token, Html(html)).into_response();
    *response.status_mut() = StatusCode::OK;
    Ok(response)
}

fn parse_user_role(input: &str) -> Result<UserRole, AppError> {
    match input {
        "admin" => Ok(UserRole::Admin),
        "instructor" => Ok(UserRole::Instructor),
        "ta" => Ok(UserRole::Ta),
        "student" => Ok(UserRole::Student),
        _ => Err(AppError::ValidationError("Invalid role".into())),
    }
}
