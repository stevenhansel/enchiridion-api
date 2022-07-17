use std::sync::Arc;

use actix_web::{HttpResponse, web};
use serde::Serialize;

use super::{RoleError, RoleServiceInterface};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRoleResponse {
    contents: Vec<ListRoleContent>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRoleContent {
    id: i32,
    name: String,
    description: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    message: String,
}

pub async fn list_role(role_service: web::Data<Arc<dyn RoleServiceInterface + Send + Sync + 'static>>) -> HttpResponse {
    let roles = match role_service.get_list_role().await {
        Ok(roles) => roles,
        Err(e) => {
            match e {
                RoleError::InternalServerError(message) => {
                    return HttpResponse::InternalServerError().json(ErrorResponse { message })
                }
            };
        }
    };

    let mut contents: Vec<ListRoleContent> = vec![];
    for role in roles {
        contents.push(ListRoleContent {
            id: role.id,
            name: role.name,
            description: role.description,
        });
    }

    HttpResponse::Ok().json(ListRoleResponse { contents })
}
