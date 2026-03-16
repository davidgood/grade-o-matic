use axum::response::Html;
use axum::{
    Extension,
    extract::{Form, FromRef, Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum_csrf::CsrfToken;
use chrono::{DateTime, NaiveDateTime, Utc};
use minijinja::context;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::common::jwt::Claims;
use crate::common::multipart_helper::parse_multipart_to_maps;
use crate::domains::assignments::dto::assignment_dto::UpdateAssignmentDto;
use crate::domains::assignments::{AssignmentDeadlineType, AssignmentServiceTrait};
use crate::domains::class_memberships::{
    ClassMembershipRole, ClassMembershipServiceTrait,
    dto::class_membership_dto::CreateClassMembershipDto,
};
use crate::domains::classes::ClassServiceTrait;
use crate::domains::classes::dto::class_dto::{CreateClassDto, UpdateClassDto};
use crate::domains::file::{FileServiceTrait, dto::file_dto::UploadFileDto};
use crate::domains::user::{UserAssetPattern, UserRole, UserServiceTrait};

use super::render_template;

#[derive(Clone)]
pub struct InstructorAttachmentDeps {
    assignment_service: Arc<dyn AssignmentServiceTrait>,
    class_service: Arc<dyn ClassServiceTrait>,
    file_service: Arc<dyn FileServiceTrait>,
    asset_pattern: UserAssetPattern,
}

impl<S> FromRef<S> for InstructorAttachmentDeps
where
    Arc<dyn AssignmentServiceTrait>: FromRef<S>,
    Arc<dyn ClassServiceTrait>: FromRef<S>,
    Arc<dyn FileServiceTrait>: FromRef<S>,
    UserAssetPattern: FromRef<S>,
{
    fn from_ref(input: &S) -> Self {
        Self {
            assignment_service: Arc::<dyn AssignmentServiceTrait>::from_ref(input),
            class_service: Arc::<dyn ClassServiceTrait>::from_ref(input),
            file_service: Arc::<dyn FileServiceTrait>::from_ref(input),
            asset_pattern: UserAssetPattern::from_ref(input),
        }
    }
}

#[derive(Debug, serde::Serialize)]
struct InstructorStudentSubmissionRow {
    file_id: Uuid,
    origin_file_name: String,
    content_type: String,
    file_size: i64,
    submitted_at: DateTime<Utc>,
    is_late: bool,
    grading_status: String,
    grading_completed_at: Option<DateTime<Utc>>,
}

fn parse_deadline_type(value: &str) -> Result<AssignmentDeadlineType, AppError> {
    match value {
        "hard_cutoff" => Ok(AssignmentDeadlineType::HardCutoff),
        "soft_deadline" => Ok(AssignmentDeadlineType::SoftDeadline),
        _ => Err(AppError::ValidationError("Invalid deadline type".into())),
    }
}

pub async fn instructors_page(
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
    Extension(claims): Extension<Claims>,
) -> Result<Html<String>, AppError> {
    let classes = class_service.list().await?;
    let owned_classes = classes
        .into_iter()
        .filter(|class| class.owner_id == Some(claims.sub))
        .collect::<Vec<_>>();

    let html = render_template(
        "classes/index.html",
        context! {
            title => "Instructors",
            classes => owned_classes,
        },
    )?;
    Ok(Html(html))
}

pub async fn instructor_class_detail_page(
    Path(class_id): Path<Uuid>,
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
    State(assignment_service): State<Arc<dyn AssignmentServiceTrait>>,
    State(class_membership_service): State<Arc<dyn ClassMembershipServiceTrait>>,
    State(user_service): State<Arc<dyn UserServiceTrait>>,
    Extension(claims): Extension<Claims>,
    token: CsrfToken,
) -> Result<Response, AppError> {
    let class = class_service
        .find_by_id(class_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Class not found".into()))?;

    // Allow owner instructors and admins to access class details.
    if !matches!(claims.user_role, UserRole::Admin) && class.owner_id != Some(claims.sub) {
        return Err(AppError::Forbidden);
    }

    let assignments = assignment_service
        .list_by_class_with_attachment_count(class_id)
        .await?;

    let memberships = class_membership_service.list_by_class_id(class_id).await?;
    let users = user_service.get_users().await?;
    let user_by_id: HashMap<Uuid, _> = users.iter().map(|u| (u.id, u)).collect();

    let roster_members = memberships
        .iter()
        .filter_map(|membership| {
            user_by_id.get(&membership.user_id).map(|user| {
                context! {
                    membership_id => membership.id,
                    user_id => user.id,
                    username => user.username.clone(),
                    email => user.email.clone(),
                    role => format!("{:?}", membership.role).to_lowercase(),
                }
            })
        })
        .collect::<Vec<_>>();

    let enrolled_user_ids: HashSet<Uuid> = memberships.iter().map(|m| m.user_id).collect();
    let available_students = users
        .into_iter()
        .filter(|user| matches!(user.user_role, UserRole::Student))
        .filter(|user| !enrolled_user_ids.contains(&user.id))
        .map(|user| {
            context! {
                id => user.id,
                username => user.username,
                email => user.email,
            }
        })
        .collect::<Vec<_>>();

    let html = render_template(
        "classes/class_detail.html",
        context! {
            title => class.title.clone(),
            class => class,
            assignments => assignments,
            roster_members => roster_members,
            available_students => available_students,
            authenticity_token => token.authenticity_token().unwrap_or_default(),
        },
    )?;
    Ok((token, Html(html)).into_response())
}

pub async fn create_class_page(
    Extension(claims): Extension<Claims>,
    token: CsrfToken,
) -> Result<Response, AppError> {
    if !matches!(claims.user_role, UserRole::Instructor | UserRole::Admin) {
        return Err(AppError::Forbidden);
    }

    let authenticity_token = token
        .authenticity_token()
        .map_err(|_| AppError::InternalError)?;

    let html = render_template(
        "classes/create_class.html",
        context! {
            title => "Create Class",
            error => "",
            class => None::<crate::domains::classes::dto::class_dto::ClassDto>,
            form_action => "/ui/instructors/classes/new",
            title_value => "",
            term_value => "",
            description_value => "",
            authenticity_token => authenticity_token,
        },
    )?;
    Ok((token, Html(html)).into_response())
}

#[derive(serde::Deserialize)]
pub struct AddRosterStudentForm {
    student_user_id: Uuid,
    authenticity_token: String,
}

#[derive(serde::Deserialize)]
pub struct RemoveRosterStudentForm {
    authenticity_token: String,
}

pub async fn add_student_to_roster(
    Path(class_id): Path<Uuid>,
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
    State(class_membership_service): State<Arc<dyn ClassMembershipServiceTrait>>,
    State(user_service): State<Arc<dyn UserServiceTrait>>,
    Extension(claims): Extension<Claims>,
    token: CsrfToken,
    Form(form): Form<AddRosterStudentForm>,
) -> Result<Response, AppError> {
    if !matches!(claims.user_role, UserRole::Instructor | UserRole::Admin) {
        return Err(AppError::Forbidden);
    }

    if token.verify(&form.authenticity_token).is_err() {
        return Err(AppError::Forbidden);
    }

    let class = class_service
        .find_by_id(class_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Class not found".into()))?;

    if !matches!(claims.user_role, UserRole::Admin) && class.owner_id != Some(claims.sub) {
        return Err(AppError::Forbidden);
    }

    let student = user_service.get_user_by_id(form.student_user_id).await?;
    if !matches!(student.user_role, UserRole::Student) {
        return Err(AppError::ValidationError(
            "Selected user is not a student.".into(),
        ));
    }

    match class_membership_service
        .create(CreateClassMembershipDto {
            class_id,
            user_id: form.student_user_id,
            role: ClassMembershipRole::Student,
        })
        .await
    {
        Ok(_) => Ok((
            StatusCode::SEE_OTHER,
            Redirect::to(&format!("/ui/instructors/classes/{class_id}")),
        )
            .into_response()),
        Err(AppError::DatabaseError(err))
            if err.to_string().contains("duplicate key value")
                || err.to_string().contains("unique") =>
        {
            Ok((
                StatusCode::SEE_OTHER,
                Redirect::to(&format!("/ui/instructors/classes/{class_id}")),
            )
                .into_response())
        }
        Err(err) => Err(err),
    }
}

pub async fn remove_student_from_roster(
    Path((class_id, membership_id)): Path<(Uuid, Uuid)>,
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
    State(class_membership_service): State<Arc<dyn ClassMembershipServiceTrait>>,
    Extension(claims): Extension<Claims>,
    token: CsrfToken,
    Form(form): Form<RemoveRosterStudentForm>,
) -> Result<Response, AppError> {
    if !matches!(claims.user_role, UserRole::Instructor | UserRole::Admin) {
        return Err(AppError::Forbidden);
    }

    if token.verify(&form.authenticity_token).is_err() {
        return Err(AppError::Forbidden);
    }

    let class = class_service
        .find_by_id(class_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Class not found".into()))?;
    if !matches!(claims.user_role, UserRole::Admin) && class.owner_id != Some(claims.sub) {
        return Err(AppError::Forbidden);
    }

    let membership = class_membership_service
        .find_by_id(membership_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Roster membership not found".into()))?;

    if membership.class_id != class_id {
        return Err(AppError::Forbidden);
    }

    class_membership_service.delete(membership_id).await?;
    Ok((
        StatusCode::SEE_OTHER,
        Redirect::to(&format!("/ui/instructors/classes/{class_id}")),
    )
        .into_response())
}

pub async fn edit_class_page(
    Path(class_id): Path<Uuid>,
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
    Extension(claims): Extension<Claims>,
    token: CsrfToken,
) -> Result<Response, AppError> {
    if !matches!(claims.user_role, UserRole::Instructor | UserRole::Admin) {
        return Err(AppError::Forbidden);
    }

    let class = class_service
        .find_by_id(class_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Class not found".into()))?;

    if !matches!(claims.user_role, UserRole::Admin) && class.owner_id != Some(claims.sub) {
        return Err(AppError::Forbidden);
    }

    let authenticity_token = token
        .authenticity_token()
        .map_err(|_| AppError::InternalError)?;

    let html = render_template(
        "classes/create_class.html",
        context! {
            title => "Edit Class",
            error => "",
            class => class,
            form_action => format!("/ui/instructors/classes/{class_id}/edit"),
            title_value => "",
            term_value => "",
            description_value => "",
            authenticity_token => authenticity_token,
        },
    )?;
    Ok((token, Html(html)).into_response())
}

#[derive(serde::Deserialize)]
pub struct CreateClassForm {
    title: String,
    description: Option<String>,
    term: Option<String>,
    authenticity_token: String,
}

pub async fn create_class_submit(
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
    Extension(claims): Extension<Claims>,
    token: CsrfToken,
    Form(form): Form<CreateClassForm>,
) -> Result<Response, AppError> {
    if !matches!(claims.user_role, UserRole::Instructor | UserRole::Admin) {
        return Err(AppError::Forbidden);
    }

    if token.verify(&form.authenticity_token).is_err() {
        return Err(AppError::Forbidden);
    }

    let title = form.title.trim().to_string();
    let term_value = form.term.as_deref().unwrap_or("").trim().to_string();
    let description_value = form.description.as_deref().unwrap_or("").trim().to_string();

    if title.is_empty() {
        let html = render_template(
            "classes/create_class.html",
            context! {
                title => "Create Class",
                error => "Title is required.",
                title_value => "",
                term_value => term_value.clone(),
                description_value => description_value.clone(),
                authenticity_token => token.authenticity_token().unwrap_or_default(),
            },
        )?;
        let mut response = (token, Html(html)).into_response();
        *response.status_mut() = StatusCode::BAD_REQUEST;
        return Ok(response);
    }

    let payload = CreateClassDto {
        title,
        description: if description_value.is_empty() {
            None
        } else {
            Some(description_value.clone())
        },
        term: if term_value.is_empty() {
            None
        } else {
            Some(term_value.clone())
        },
        owner_id: Some(claims.sub),
        modified_by: claims.sub,
    };

    match class_service.create(payload).await {
        Ok(created) => Ok((
            StatusCode::SEE_OTHER,
            Redirect::to(&format!("/ui/instructors/classes/{}", created.id)),
        )
            .into_response()),
        Err(err) => {
            let html = render_template(
                "classes/create_class.html",
                context! {
                    title => "Create Class",
                    error => err.to_string(),
                    title_value => form.title,
                    term_value => term_value,
                    description_value => description_value,
                    authenticity_token => token.authenticity_token().unwrap_or_default(),
                },
            )?;
            let mut response = (token, Html(html)).into_response();
            *response.status_mut() = StatusCode::BAD_REQUEST;
            Ok(response)
        }
    }
}

pub async fn edit_class_submit(
    Path(class_id): Path<Uuid>,
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
    Extension(claims): Extension<Claims>,
    token: CsrfToken,
    Form(form): Form<CreateClassForm>,
) -> Result<Response, AppError> {
    if !matches!(claims.user_role, UserRole::Instructor | UserRole::Admin) {
        return Err(AppError::Forbidden);
    }

    if token.verify(&form.authenticity_token).is_err() {
        return Err(AppError::Forbidden);
    }

    let existing = class_service
        .find_by_id(class_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Class not found".into()))?;

    if !matches!(claims.user_role, UserRole::Admin) && existing.owner_id != Some(claims.sub) {
        return Err(AppError::Forbidden);
    }

    let title = form.title.trim().to_string();
    let term_value = form.term.as_deref().unwrap_or("").trim().to_string();
    let description_value = form.description.as_deref().unwrap_or("").trim().to_string();

    if title.is_empty() {
        let html = render_template(
            "classes/create_class.html",
            context! {
                title => "Edit Class",
                error => "Title is required.",
                class => existing,
                form_action => format!("/ui/instructors/classes/{class_id}/edit"),
                title_value => title,
                term_value => term_value,
                description_value => description_value,
                authenticity_token => token.authenticity_token().unwrap_or_default(),
            },
        )?;
        let mut response = (token, Html(html)).into_response();
        *response.status_mut() = StatusCode::BAD_REQUEST;
        return Ok(response);
    }

    let payload = UpdateClassDto {
        id: class_id,
        title,
        description: if description_value.is_empty() {
            None
        } else {
            Some(description_value.clone())
        },
        term: if term_value.is_empty() {
            None
        } else {
            Some(term_value.clone())
        },
        owner_id: existing.owner_id,
        modified_by: claims.sub,
    };

    match class_service.update(payload).await {
        Ok(Some(updated)) => Ok((
            StatusCode::SEE_OTHER,
            Redirect::to(&format!("/ui/instructors/classes/{}", updated.id)),
        )
            .into_response()),
        Ok(None) => Err(AppError::NotFound("Class not found".into())),
        Err(err) => {
            let html = render_template(
                "classes/create_class.html",
                context! {
                    title => "Edit Class",
                    error => err.to_string(),
                    class => existing,
                    form_action => format!("/ui/instructors/classes/{class_id}/edit"),
                    title_value => "",
                    term_value => term_value,
                    description_value => description_value,
                    authenticity_token => token.authenticity_token().unwrap_or_default(),
                },
            )?;
            let mut response = (token, Html(html)).into_response();
            *response.status_mut() = StatusCode::BAD_REQUEST;
            Ok(response)
        }
    }
}

pub async fn edit_assignment_page(
    Path(assignment_id): Path<Uuid>,
    State(assignment_service): State<Arc<dyn AssignmentServiceTrait>>,
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
    State(class_membership_service): State<Arc<dyn ClassMembershipServiceTrait>>,
    State(user_service): State<Arc<dyn UserServiceTrait>>,

    Extension(claims): Extension<Claims>,
    token: CsrfToken,
) -> Result<Response, AppError> {
    if !matches!(claims.user_role, UserRole::Instructor | UserRole::Admin) {
        return Err(AppError::Forbidden);
    }

    let assignment = assignment_service
        .find_by_id(assignment_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Assignment not found".into()))?;

    let class = class_service
        .find_by_id(assignment.class_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Class not found".into()))?;

    if !matches!(claims.user_role, UserRole::Admin) && class.owner_id != Some(claims.sub) {
        return Err(AppError::Forbidden);
    }

    let memberships = class_membership_service.list_by_class_id(class.id).await?;
    let users = user_service.get_users().await?;
    let user_by_id: HashMap<Uuid, _> = users.iter().map(|user| (user.id, user)).collect();
    let roster_members = memberships
        .iter()
        .filter(|membership| matches!(membership.role, ClassMembershipRole::Student))
        .filter_map(|membership| {
            user_by_id.get(&membership.user_id).map(|user| {
                context! {
                    user_id => user.id,
                    username => user.username.clone(),
                    email => user.email.clone(),
                }
            })
        })
        .collect::<Vec<_>>();

    let authenticity_token = token
        .authenticity_token()
        .map_err(|_| AppError::InternalError)?;

    let html = render_template(
        "assignments/create_assignment.html",
        context! {
            title => "Edit Assignment",
            error => "",
            assignment => assignment,
            class => class,
            attachments => assignment_service.list_attachments(assignment_id).await?,
            roster_members => roster_members,
            upload_message => "",
            upload_error => "",
            form_action => format!("/ui/instructors/assignments/{assignment_id}/edit"),
            title_value => assignment.title.clone(),
            description_value => assignment.description.clone(),
            due_at_value => assignment.due_at,
            deadline_type_value => assignment.deadline_type,
            points_value => assignment.points,
            authenticity_token => authenticity_token,
        },
    )?;
    Ok((token, Html(html)).into_response())
}

pub async fn instructor_student_submission_history_page(
    Path((assignment_id, student_id)): Path<(Uuid, Uuid)>,
    State(assignment_service): State<Arc<dyn AssignmentServiceTrait>>,
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
    State(class_membership_service): State<Arc<dyn ClassMembershipServiceTrait>>,
    State(user_service): State<Arc<dyn UserServiceTrait>>,
    Extension(claims): Extension<Claims>,
) -> Result<Html<String>, AppError> {
    if !matches!(claims.user_role, UserRole::Instructor | UserRole::Admin) {
        return Err(AppError::Forbidden);
    }

    let assignment = assignment_service
        .find_by_id(assignment_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Assignment not found".into()))?;

    let class = class_service
        .find_by_id(assignment.class_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Class not found".into()))?;

    if !matches!(claims.user_role, UserRole::Admin) && class.owner_id != Some(claims.sub) {
        return Err(AppError::Forbidden);
    }

    let memberships = class_membership_service.list_by_class_id(class.id).await?;
    let is_enrolled = memberships.iter().any(|membership| {
        membership.user_id == student_id && matches!(membership.role, ClassMembershipRole::Student)
    });
    if !is_enrolled {
        return Err(AppError::NotFound(
            "Student is not enrolled in this class".into(),
        ));
    }

    let student = user_service.get_user_by_id(student_id).await?;
    if !matches!(student.user_role, UserRole::Student) {
        return Err(AppError::NotFound("Student not found".into()));
    }

    let submissions = assignment_service
        .list_student_submission_history(assignment_id, student_id)
        .await?
        .into_iter()
        .map(|submission| InstructorStudentSubmissionRow {
            file_id: submission.file_id,
            origin_file_name: submission.origin_file_name,
            content_type: submission.content_type,
            file_size: submission.file_size,
            submitted_at: submission.submitted_at,
            is_late: submission.is_late,
            grading_status: submission
                .grading_status
                .unwrap_or_else(|| "not_queued".to_string()),
            grading_completed_at: submission.grading_completed_at,
        })
        .collect::<Vec<_>>();

    let html = render_template(
        "assignments/instructor_submission_history.html",
        context! {
            title => format!("Submission History: {}", assignment.title),
            class => class,
            assignment => assignment,
            student => student,
            submissions => submissions,
        },
    )?;
    Ok(Html(html))
}

pub async fn edit_assignment_submit(
    Path(assignment_id): Path<Uuid>,
    State(assignment_service): State<Arc<dyn AssignmentServiceTrait>>,
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
    Extension(claims): Extension<Claims>,
    token: CsrfToken,
    Form(form): Form<EditAssignmentForm>,
) -> Result<Response, AppError> {
    if !matches!(claims.user_role, UserRole::Instructor | UserRole::Admin) {
        return Err(AppError::Forbidden);
    }

    if token.verify(&form.authenticity_token).is_err() {
        return Err(AppError::Forbidden);
    }

    let assignment = assignment_service
        .find_by_id(assignment_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Assignment not found".into()))?;

    let existing = class_service
        .find_by_id(assignment.class_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Class not found".into()))?;

    if !matches!(claims.user_role, UserRole::Admin) && existing.owner_id != Some(claims.sub) {
        return Err(AppError::Forbidden);
    }

    let attachments = assignment_service.list_attachments(assignment_id).await?;

    let title = form.title.trim().to_string();
    let description_value = form.description.as_deref().unwrap_or("").trim().to_string();
    let points = form.points;
    let due_at_value = form.due_at.as_deref().unwrap_or("").trim().to_string();
    let deadline_type_value = form.deadline_type.trim().to_string();

    if title.is_empty() {
        let html = render_template(
            "assignments/create_assignment.html",
            context! {
                title => "Edit Assignment",
                error => "Title is required.",
                assignment => assignment,
                class => existing,
                attachments => attachments.clone(),
                upload_message => "",
                upload_error => "",
                form_action => format!("/ui/instructors/assignments/{assignment_id}/edit"),
                title_value => title,
                description_value => description_value,
                due_at_value => due_at_value,
                deadline_type_value => deadline_type_value,
                points_value => points,
                authenticity_token => token.authenticity_token().unwrap_or_default(),
            },
        )?;
        let mut response = (token, Html(html)).into_response();
        *response.status_mut() = StatusCode::BAD_REQUEST;
        return Ok(response);
    }

    let payload = UpdateAssignmentDto {
        id: assignment_id,
        class_id: assignment.class_id,
        title,
        description: if description_value.is_empty() {
            None
        } else {
            Some(description_value.clone())
        },
        due_at: if due_at_value.is_empty() {
            None
        } else {
            let parsed = NaiveDateTime::parse_from_str(&due_at_value, "%Y-%m-%dT%H:%M")
                .map_err(|_| AppError::ValidationError("Invalid due date format".into()))?;
            Some(DateTime::<Utc>::from_naive_utc_and_offset(parsed, Utc))
        },
        deadline_type: parse_deadline_type(&deadline_type_value)?,
        points: if points.is_negative() {
            Some(0)
        } else {
            Some(points)
        },
        modified_by: claims.sub,
    };

    match assignment_service.update(payload).await {
        Ok(updated) => Ok((
            StatusCode::SEE_OTHER,
            Redirect::to(&format!("/ui/instructors/classes/{}", updated.class_id)),
        )
            .into_response()),
        Err(err) => {
            let html = render_template(
                "assignments/create_assignment.html",
                context! {
                    title => "Edit Assignment",
                    error => err.to_string(),
                    assignment => assignment,
                    class => existing,
                    attachments => attachments,
                    upload_message => "",
                    upload_error => "",
                    form_action => format!("/ui/instructors/assignments/{assignment_id}/edit"),
                    title_value => "",
                    due_at_value => due_at_value,
                    deadline_type_value => deadline_type_value,
                    description_value => description_value,
                    points_value => points,
                    authenticity_token => token.authenticity_token().unwrap_or_default(),
                },
            )?;
            let mut response = (token, Html(html)).into_response();
            *response.status_mut() = StatusCode::BAD_REQUEST;
            Ok(response)
        }
    }
}

pub async fn upload_assignment_attachments(
    Path(assignment_id): Path<Uuid>,
    State(deps): State<InstructorAttachmentDeps>,
    Extension(claims): Extension<Claims>,
    token: CsrfToken,
    multipart: Multipart,
) -> Result<Response, AppError> {
    if !matches!(claims.user_role, UserRole::Instructor | UserRole::Admin) {
        return Err(AppError::Forbidden);
    }

    let assignment = deps
        .assignment_service
        .find_by_id(assignment_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Assignment not found".into()))?;

    let class = deps
        .class_service
        .find_by_id(assignment.class_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Class not found".into()))?;

    if !matches!(claims.user_role, UserRole::Admin) && class.owner_id != Some(claims.sub) {
        return Err(AppError::Forbidden);
    }

    let (mut fields, mut files) = parse_multipart_to_maps(multipart, &deps.asset_pattern.0).await?;
    let csrf_token = fields
        .remove("authenticity_token")
        .ok_or_else(|| AppError::ValidationError("Missing authenticity token".into()))?;

    if token.verify(&csrf_token).is_err() {
        return Err(AppError::Forbidden);
    }

    let attachments = files.remove("attachments").unwrap_or_default();
    if attachments.is_empty() {
        let panel = render_template(
            "assignments/partials/_attachments_panel.html",
            context! {
                assignment => assignment,
                attachments => deps.assignment_service.list_attachments(assignment_id).await?,
                authenticity_token => token.authenticity_token().unwrap_or_default(),
                upload_message => "",
                upload_error => "Please choose at least one file.",
            },
        )?;
        let mut response = (token, Html(panel)).into_response();
        *response.status_mut() = StatusCode::BAD_REQUEST;
        return Ok(response);
    }

    let upload_count = attachments.len();
    for file in attachments {
        let upload = UploadFileDto {
            file,
            user_id: Some(claims.sub),
            modified_by: claims.sub,
        };
        let uploaded = match deps
            .file_service
            .process_assignment_file_upload(&upload)
            .await
        {
            Ok(uploaded) => uploaded,
            Err(err) => {
                let panel = render_template(
                    "assignments/partials/_attachments_panel.html",
                    context! {
                        assignment => assignment,
                        attachments => deps.assignment_service.list_attachments(assignment_id).await?,
                        authenticity_token => token.authenticity_token().unwrap_or_default(),
                        upload_message => "",
                        upload_error => err.to_string(),
                    },
                )?;
                let mut response = (token, Html(panel)).into_response();
                *response.status_mut() = StatusCode::BAD_REQUEST;
                return Ok(response);
            }
        };
        deps.assignment_service
            .attach_file(assignment_id, uploaded.id, claims.sub)
            .await?;
    }

    let panel = render_template(
        "assignments/partials/_attachments_panel.html",
        context! {
            assignment => assignment,
            attachments => deps.assignment_service.list_attachments(assignment_id).await?,
            authenticity_token => token.authenticity_token().unwrap_or_default(),
            upload_message => format!("Uploaded {} file(s).", upload_count),
            upload_error => "",
        },
    )?;
    Ok((token, Html(panel)).into_response())
}

#[derive(serde::Deserialize)]
pub struct EditAssignmentForm {
    title: String,
    description: Option<String>,
    due_at: Option<String>,
    deadline_type: String,
    authenticity_token: String,
    points: i16,
}
