use std::sync::Arc;

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::http::{
    derive_authentication_middleware_error, derive_user_id, AuthenticationContext,
    HttpErrorResponse,
};

use super::{
    ListRequestError, ListRequestParams, RequestActionType, RequestErrorCode,
    RequestServiceInterface, UpdateRequestApprovalError, UpdateRequestApprovalParams,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRequestQueryParams {
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub request_id: Option<i32>,
    pub announcement_id: Option<i32>,
    pub user_id: Option<i32>,
    pub action_type: Option<RequestActionType>,
    pub approved_by_lsc: Option<bool>,
    pub approved_by_bm: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRequestResponse {
    count: i32,
    total_pages: i32,
    has_next: bool,
    contents: Vec<ListRequestContent>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRequestContent {
    id: i32,
    announcement: ListRequestContentAnnouncement,
    author: ListRequestContentAuthor,
    approval_status: ListRequestContentApprovalStatus,
    action: ListRequestContentAction,
    description: String,
    created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRequestContentAnnouncement {
    id: i32,
    title: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRequestContentAuthor {
    id: i32,
    name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRequestContentApprovalStatus {
    lsc: Option<bool>,
    bm: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRequestContentAction {
    label: String,
    value: String,
}

pub async fn list_request(
    request_service: web::Data<Arc<dyn RequestServiceInterface + Send + Sync + 'static>>,
    auth: AuthenticationContext,
    query_params: web::Query<ListRequestQueryParams>,
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

    let result = match request_service
        .list_request(ListRequestParams {
            page,
            limit,
            request_id: query_params.request_id,
            announcement_id: query_params.announcement_id,
            user_id: query_params.user_id,
            action_type: query_params.action_type.clone(),
            approved_by_lsc: query_params.approved_by_lsc,
            approved_by_bm: query_params.approved_by_bm,
        })
        .await
    {
        Ok(request) => request,
        Err(e) => match e {
            ListRequestError::InternalServerError => {
                return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                    RequestErrorCode::InternalServerError.to_string(),
                    vec![ListRequestError::InternalServerError.to_string()],
                ))
            }
        },
    };

    HttpResponse::Ok().json(ListRequestResponse {
        count: result.count,
        total_pages: result.total_pages,
        has_next: result.has_next,
        contents: result
            .contents
            .into_iter()
            .map(|row| ListRequestContent {
                id: row.id,
                announcement: ListRequestContentAnnouncement {
                    id: row.announcement_id,
                    title: row.announcement_title,
                },
                author: ListRequestContentAuthor {
                    id: row.user_id,
                    name: row.user_name,
                },
                approval_status: ListRequestContentApprovalStatus {
                    lsc: row.approved_by_lsc,
                    bm: row.approved_by_bm,
                },
                action: ListRequestContentAction {
                    label: row.action.clone().label().to_string(),
                    value: row.action.clone().value().to_string(),
                },
                description: row.description,
                created_at: row.created_at.to_rfc3339(),
            })
            .collect(),
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRequestApprovalBody {
    action: String,
}

pub async fn update_request_approval(
    request_service: web::Data<Arc<dyn RequestServiceInterface + Send + Sync + 'static>>,
    auth: AuthenticationContext,
    body: web::Json<UpdateRequestApprovalBody>,
    request_id: web::Path<i32>,
) -> HttpResponse {
    let request_id = request_id.into_inner();
    let user_id = match derive_user_id(auth) {
        Ok(id) => id,
        Err(e) => return derive_authentication_middleware_error(e),
    };
    let approval = match body.action.as_str() {
        "approve" => true,
        "reject" => false,
        _ => {
            return HttpResponse::BadRequest().json(HttpErrorResponse::new(
                "API_VALIDATION_ERROR".into(),
                vec!["action should be approve or reject".into()],
            ))
        }
    };

    if let Err(e) = request_service
        .update_request_approval(UpdateRequestApprovalParams {
            request_id,
            approval,
            approver_id: user_id,
        })
        .await
    {
        match e {
            UpdateRequestApprovalError::RequestNotFound(message) => {
                return HttpResponse::NotFound().json(HttpErrorResponse::new(
                    RequestErrorCode::RequestNotFound.to_string(),
                    vec![message],
                ))
            }
            UpdateRequestApprovalError::UserNotFound(message) => {
                return HttpResponse::NotFound().json(HttpErrorResponse::new(
                    RequestErrorCode::UserNotFound.to_string(),
                    vec![message],
                ))
            }
            UpdateRequestApprovalError::AnnouncementNotFound(message) => {
                return HttpResponse::NotFound().json(HttpErrorResponse::new(
                    RequestErrorCode::AnnouncementNotFound.to_string(),
                    vec![message],
                ))
            }
            UpdateRequestApprovalError::UserForbiddenToApprove(message) => {
                return HttpResponse::Conflict().json(HttpErrorResponse::new(
                    RequestErrorCode::UserForbiddenToApprove.to_string(),
                    vec![message],
                ))
            }
            UpdateRequestApprovalError::RequestAlreadyApproved(message) => {
                return HttpResponse::Conflict().json(HttpErrorResponse::new(
                    RequestErrorCode::RequestAlreadyApproved.to_string(),
                    vec![message],
                ))
            }
            UpdateRequestApprovalError::InvalidAnnouncementStatus(message) => {
                return HttpResponse::Conflict().json(HttpErrorResponse::new(
                    RequestErrorCode::InvalidAnnouncementStatus.to_string(),
                    vec![message],
                ))
            }
            UpdateRequestApprovalError::InternalServerError => {
                return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                    RequestErrorCode::InternalServerError.to_string(),
                    vec![UpdateRequestApprovalError::InternalServerError.to_string()],
                ))
            }
        }
    }

    HttpResponse::NoContent().finish()
}
