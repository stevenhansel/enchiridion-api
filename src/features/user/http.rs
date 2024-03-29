use std::sync::Arc;

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::{
    database::PaginationResult,
    http::{
        derive_authentication_middleware_error, derive_user_id, AuthenticationContext,
        HttpErrorResponse, API_VALIDATION_ERROR_CODE,
    },
};

use super::{
    ListUserError, ListUserParams, UpdateUserApprovalError, UserErrorCode, UserServiceInterface,
    UserStatus,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListUserQueryParams {
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub query: Option<String>,
    pub status: Option<UserStatus>,
    pub role: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListUserContentResponse {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub role: ListUserContentRole,
    pub status: ListUserContentStatus,
    pub is_email_confirmed: bool,
    pub registration_reason: Option<String>,
    pub building: Option<ListUserBuilding>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListUserContentRole {
    pub name: String,
    pub value: String,
    pub description: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListUserContentStatus {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListUserBuilding {
    pub id: i32,
    pub name: String,
    pub color: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_user(
    user_service: web::Data<Arc<dyn UserServiceInterface + Send + Sync + 'static>>,
    auth: AuthenticationContext,
    query_params: web::Query<ListUserQueryParams>,
) -> HttpResponse {
    if let Err(e) = derive_user_id(auth) {
        return derive_authentication_middleware_error(e);
    }

    let mut page = 1;
    if let Some(raw_page) = query_params.page {
        page = raw_page;
    }

    let mut limit = 25;
    if let Some(raw_limit) = query_params.limit {
        limit = raw_limit;
    }

    let result = match user_service
        .list_user(ListUserParams {
            page,
            limit,
            role: query_params.role.clone(),
            query: query_params.query.clone(),
            status: query_params.status.clone(),
        })
        .await
    {
        Ok(result) => result,
        Err(e) => match e {
            ListUserError::InternalServerError => {
                return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                    UserErrorCode::InternalServerError.to_string(),
                    vec![ListUserError::InternalServerError.to_string()],
                ))
            }
        },
    };

    let response: PaginationResult<ListUserContentResponse> = PaginationResult {
        count: result.count,
        total_pages: result.total_pages,
        has_next: result.has_next,
        contents: result
            .contents
            .into_iter()
            .map(|u| ListUserContentResponse {
                id: u.id,
                name: u.name,
                email: u.email,
                role: ListUserContentRole {
                    name: u.role.name.to_string(),
                    value: u.role.value.to_string(),
                    description: u.role.description.to_string(),
                },
                status: ListUserContentStatus {
                    label: u.status.clone().label().to_string(),
                    value: u.status.clone().value().to_string(),
                },
                is_email_confirmed: u.is_email_confirmed,
                registration_reason: u.registration_reason,
                building: if let Some(b) = u.building {
                    Some(ListUserBuilding {
                        id: b.id,
                        name: b.name,
                        color: b.color,
                        created_at: b.created_at,
                        updated_at: b.updated_at,
                    })
                } else {
                    None
                },
            })
            .collect(),
    };

    HttpResponse::Ok().json(response)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserApprovalBody {
    action: UserApprovalAction,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UserApprovalAction {
    Approve,
    Reject,
}

pub async fn update_user_approval(
    user_service: web::Data<Arc<dyn UserServiceInterface + Send + Sync + 'static>>,
    auth: AuthenticationContext,
    path: web::Path<String>,
    body: web::Json<UpdateUserApprovalBody>,
) -> HttpResponse {
    if let Err(e) = derive_user_id(auth) {
        return derive_authentication_middleware_error(e);
    }

    let user_id: i32 = match path.into_inner().parse() {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(HttpErrorResponse::new(
                String::from(API_VALIDATION_ERROR_CODE),
                vec!["Invalid user id, must be a valid integer".into()],
            ))
        }
    };

    if let Err(e) = user_service
        .update_user_approval(
            user_id,
            match &body.action {
                UserApprovalAction::Approve => true,
                UserApprovalAction::Reject => false,
            },
        )
        .await
    {
        match e {
            UpdateUserApprovalError::UserNotFound(message) => {
                return HttpResponse::NotFound().json(HttpErrorResponse::new(
                    UserErrorCode::UserNotFound.to_string(),
                    vec![message.into()],
                ))
            }
            UpdateUserApprovalError::UserNotConfirmed(message) => {
                return HttpResponse::Conflict().json(HttpErrorResponse::new(
                    UserErrorCode::UserStatusConflict.to_string(),
                    vec![message.into()],
                ))
            }
            UpdateUserApprovalError::UserStatusConflict(message) => {
                return HttpResponse::Conflict().json(HttpErrorResponse::new(
                    UserErrorCode::UserStatusConflict.to_string(),
                    vec![message.into()],
                ))
            }
            UpdateUserApprovalError::InternalServerError => {
                return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                    UserErrorCode::InternalServerError.to_string(),
                    vec![UpdateUserApprovalError::InternalServerError.to_string()],
                ))
            }
        }
    }

    HttpResponse::NoContent().finish()
}
