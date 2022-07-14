use actix_web::HttpResponse;
use serde::Serialize;
use shaku_actix::Inject;

use super::RoleServiceInterface;
use crate::container::Container;

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


pub async fn list_role(role_service: Inject<Container, dyn RoleServiceInterface>) -> HttpResponse {
    let roles = match role_service.get_list_role().await {
        Ok(roles) => roles,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                message: e.to_string(),
            });
        }
    };
    let mut contents: Vec<ListRoleContent> = vec![];
    for role in roles {
        contents.push(ListRoleContent{
            id: role.id,
            name: role.name,
            description: role.description,
        });
    }  

    HttpResponse::Ok().json(ListRoleResponse{contents})
}
