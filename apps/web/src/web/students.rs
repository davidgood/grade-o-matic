use crate::common::error::AppError;
use crate::common::jwt::Claims;
use crate::common::multipart_helper::parse_multipart_to_maps;
use crate::domains::assignments::AssignmentDeadlineType;
use crate::domains::assignments::AssignmentServiceTrait;
use crate::domains::class_memberships::ClassMembershipServiceTrait;
use crate::domains::classes::ClassServiceTrait;
use crate::domains::file::FileServiceTrait;
use crate::domains::file::dto::file_dto::{FileDto, UploadFileDto};
use crate::domains::user::{UserAssetPattern, UserRole};
use crate::web::render_template;
use axum::Extension;
use axum::extract::{FromRef, Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum_csrf::CsrfToken;
use chrono::Utc;
use minijinja::context;
use serde::Serialize;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Clone)]
pub struct StudentSubmissionDeps {
    class_membership_service: Arc<dyn ClassMembershipServiceTrait>,
    class_service: Arc<dyn ClassServiceTrait>,
    assignment_service: Arc<dyn AssignmentServiceTrait>,
    file_service: Arc<dyn FileServiceTrait>,
    asset_pattern: UserAssetPattern,
}
impl<S> FromRef<S> for StudentSubmissionDeps
where
    Arc<dyn ClassMembershipServiceTrait>: FromRef<S>,
    Arc<dyn ClassServiceTrait>: FromRef<S>,
    Arc<dyn AssignmentServiceTrait>: FromRef<S>,
    Arc<dyn FileServiceTrait>: FromRef<S>,
    UserAssetPattern: FromRef<S>,
{
    fn from_ref(input: &S) -> Self {
        Self {
            class_membership_service: Arc::<dyn ClassMembershipServiceTrait>::from_ref(input),
            class_service: Arc::<dyn ClassServiceTrait>::from_ref(input),
            assignment_service: Arc::<dyn AssignmentServiceTrait>::from_ref(input),
            file_service: Arc::<dyn FileServiceTrait>::from_ref(input),
            asset_pattern: UserAssetPattern::from_ref(input),
        }
    }
}

pub async fn students_classes_page(
    State(class_membership_service): State<Arc<dyn ClassMembershipServiceTrait>>,
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
    Extension(claims): Extension<Claims>,
) -> Result<Html<String>, AppError> {
    if !matches!(claims.user_role, UserRole::Student) {
        return Err(AppError::Forbidden);
    }

    let memberships = class_membership_service.list_by_user_id(claims.sub).await?;

    let mut classes = Vec::new();
    for membership in memberships {
        if let Some(class) = class_service.find_by_id(membership.class_id).await? {
            classes.push(class);
        }
    }
    classes.sort_by(|a, b| {
        a.term
            .cmp(&b.term)
            .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
    });

    let html = render_template(
        "classes/student_index.html",
        context! {
            title => "Classes",
            classes => classes,
        },
    )?;
    Ok(Html(html))
}

#[derive(Debug, Serialize)]
struct StudentAssignmentRow {
    id: uuid::Uuid,
    class_title: String,
    title: String,
    description: Option<String>,
    due_at: Option<chrono::DateTime<chrono::Utc>>,
    extension_due_at: Option<chrono::DateTime<chrono::Utc>>,
    effective_due_at: Option<chrono::DateTime<chrono::Utc>>,
    deadline_type: AssignmentDeadlineType,
    points: Option<i16>,
}

pub async fn students_assignments_page(
    State(class_membership_service): State<Arc<dyn ClassMembershipServiceTrait>>,
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
    State(assignment_service): State<Arc<dyn AssignmentServiceTrait>>,
    Extension(claims): Extension<Claims>,
) -> Result<Html<String>, AppError> {
    if !matches!(claims.user_role, UserRole::Student) {
        return Err(AppError::Forbidden);
    }

    let memberships = class_membership_service.list_by_user_id(claims.sub).await?;

    let mut rows = Vec::new();

    for membership in memberships {
        let class = match class_service.find_by_id(membership.class_id).await? {
            Some(class) => class,
            None => continue,
        };

        let assignments = assignment_service
            .list_by_class_for_student(class.id, claims.sub)
            .await?;
        for assignment in assignments {
            rows.push(StudentAssignmentRow {
                id: assignment.id,
                class_title: class.title.clone(),
                title: assignment.title,
                description: assignment.description,
                due_at: assignment.due_at,
                extension_due_at: assignment.extension_due_at,
                effective_due_at: assignment.effective_due_at,
                deadline_type: assignment.deadline_type,
                points: assignment.points,
            });
        }
    }

    rows.sort_by(|a, b| {
        a.due_at
            .cmp(&b.due_at)
            .then_with(|| {
                a.class_title
                    .to_lowercase()
                    .cmp(&b.class_title.to_lowercase())
            })
            .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
    });

    let html = render_template(
        "assignments/student_index.html",
        context! {
            title => "My Assignments",
            assignments => rows,
            count => rows.len(),
        },
    )?;
    Ok(Html(html))
}

#[derive(Debug, Serialize)]
struct StudentSubmissionAttachment {
    file_id: uuid::Uuid,
    origin_file_name: String,
    content_type: String,
    file_size: i64,
    created_at: chrono::DateTime<chrono::Utc>,
    is_late: bool,
}

async fn get_student_assignment_context(
    assignment_id: uuid::Uuid,
    claims: &Claims,
    class_membership_service: &Arc<dyn ClassMembershipServiceTrait>,
    class_service: &Arc<dyn ClassServiceTrait>,
    assignment_service: &Arc<dyn AssignmentServiceTrait>,
) -> Result<
    (
        crate::domains::assignments::dto::assignment_dto::AssignmentDto,
        crate::domains::classes::dto::class_dto::ClassDto,
        Vec<StudentSubmissionAttachment>,
    ),
    AppError,
> {
    let assignment = assignment_service
        .find_by_id_for_student(assignment_id, claims.sub)
        .await?
        .ok_or_else(|| AppError::NotFound("Assignment not found".into()))?;

    let class = class_service
        .find_by_id(assignment.class_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Class not found".into()))?;

    let memberships = class_membership_service.list_by_user_id(claims.sub).await?;
    let enrolled_class_ids: HashSet<uuid::Uuid> =
        memberships.into_iter().map(|m| m.class_id).collect();
    if !enrolled_class_ids.contains(&class.id) {
        return Err(AppError::Forbidden);
    }

    let effective_due_at = assignment.effective_due_at.or(assignment.due_at);
    let submission_files = assignment_service
        .list_attachments(assignment_id)
        .await?
        .into_iter()
        .filter(|a| a.created_by == Some(claims.sub))
        .map(|a| StudentSubmissionAttachment {
            file_id: a.file_id,
            origin_file_name: a.origin_file_name,
            content_type: a.content_type,
            file_size: a.file_size,
            created_at: a.created_at,
            is_late: matches!(
                assignment.deadline_type,
                AssignmentDeadlineType::SoftDeadline
            ) && effective_due_at.is_some_and(|due_at| a.created_at > due_at),
        })
        .collect::<Vec<_>>();

    Ok((assignment, class, submission_files))
}

#[derive(Debug, serde::Deserialize)]
pub struct StudentAssignmentDetailQuery {
    submitted: Option<String>,
}

pub async fn student_assignment_detail_page(
    Path(assignment_id): Path<uuid::Uuid>,
    Query(query): Query<StudentAssignmentDetailQuery>,
    State(class_membership_service): State<Arc<dyn ClassMembershipServiceTrait>>,
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
    State(assignment_service): State<Arc<dyn AssignmentServiceTrait>>,
    Extension(claims): Extension<Claims>,
    token: CsrfToken,
) -> Result<Response, AppError> {
    if !matches!(claims.user_role, UserRole::Student) {
        return Err(AppError::Forbidden);
    }

    let (assignment, class, submission_files) = get_student_assignment_context(
        assignment_id,
        &claims,
        &class_membership_service,
        &class_service,
        &assignment_service,
    )
    .await?;

    let html = render_template(
        "assignments/student_detail.html",
        context! {
            title => format!("Assignment: {}", assignment.title),
            class => class,
            assignment => assignment,
            submission_files => submission_files,
            submitted => query.submitted.as_deref() == Some("1"),
            error => "",
            authenticity_token => token.authenticity_token().unwrap_or_default(),
        },
    )?;
    Ok((token, Html(html)).into_response())
}

pub async fn submit_student_assignment(
    Path(assignment_id): Path<uuid::Uuid>,
    State(deps): State<StudentSubmissionDeps>,
    Extension(claims): Extension<Claims>,
    token: CsrfToken,
    multipart: Multipart,
) -> Result<Response, AppError> {
    if !matches!(claims.user_role, UserRole::Student) {
        return Err(AppError::Forbidden);
    }

    let (fields, mut files) = parse_multipart_to_maps(multipart, &deps.asset_pattern.0).await?;
    let authenticity_token = fields
        .get("authenticity_token")
        .map(std::string::String::as_str)
        .unwrap_or_default();
    if token.verify(authenticity_token).is_err() {
        return Err(AppError::Forbidden);
    }

    let (assignment, class, submission_files) = get_student_assignment_context(
        assignment_id,
        &claims,
        &deps.class_membership_service,
        &deps.class_service,
        &deps.assignment_service,
    )
    .await?;

    let mut attachments = files.remove("attachments").unwrap_or_default();
    let code_submission = fields
        .get("code_submission")
        .map(|text| text.trim().to_string())
        .unwrap_or_default();

    if !code_submission.is_empty() {
        let code_file = FileDto {
            content_type: "text/plain".to_string(),
            original_filename: format!("code-submission-{}.txt", Utc::now().timestamp_millis()),
            data: code_submission.into_bytes(),
        };
        attachments.push(code_file);
    }

    if attachments.is_empty() {
        let html = render_template(
            "assignments/student_detail.html",
            context! {
                title => format!("Assignment: {}", assignment.title),
                class => class,
                assignment => assignment,
                submission_files => submission_files,
                submitted => false,
                error => "Add code in the textarea or attach at least one file before submitting.",
                authenticity_token => token.authenticity_token().unwrap_or_default(),
            },
        )?;
        return Ok((StatusCode::BAD_REQUEST, token, Html(html)).into_response());
    }

    let effective_due_at = assignment.effective_due_at.or(assignment.due_at);
    if matches!(assignment.deadline_type, AssignmentDeadlineType::HardCutoff)
        && effective_due_at.is_some_and(|due_at| Utc::now() > due_at)
    {
        let html = render_template(
            "assignments/student_detail.html",
            context! {
                title => format!("Assignment: {}", assignment.title),
                class => class,
                assignment => assignment,
                submission_files => submission_files,
                submitted => false,
                error => "This assignment is closed. The hard cutoff deadline has passed.",
                authenticity_token => token.authenticity_token().unwrap_or_default(),
            },
        )?;
        return Ok((StatusCode::BAD_REQUEST, token, Html(html)).into_response());
    }

    for file in attachments {
        let uploaded = deps
            .file_service
            .process_assignment_file_upload(&UploadFileDto {
                file,
                user_id: Some(claims.sub),
                modified_by: claims.sub,
            })
            .await?;

        deps.assignment_service
            .attach_file(assignment_id, uploaded.id, claims.sub)
            .await?;
    }

    Ok((
        StatusCode::SEE_OTHER,
        Redirect::to(&format!(
            "/ui/students/assignments/{assignment_id}?submitted=1"
        )),
    )
        .into_response())
}
