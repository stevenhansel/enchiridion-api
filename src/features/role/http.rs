use std::sync::Arc;

use actix_web::{web, HttpResponse};
use serde::Serialize;

use super::RoleServiceInterface;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRoleResponse {
    contents: Vec<ListRoleContent>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRoleContent {
    name: &'static str,
    value: &'static str,
    description: &'static str,
}

pub async fn list_role(
    role_service: web::Data<Arc<dyn RoleServiceInterface + Send + Sync + 'static>>,
) -> HttpResponse {
    HttpResponse::Ok().json(ListRoleResponse {
        contents: role_service
            .list_role()
            .into_iter()
            .map(|r| ListRoleContent {
                name: r.name,
                value: r.value,
                description: r.description,
            })
            .collect(),
    })
}
