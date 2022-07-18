use std::sync::Arc;

use async_trait::async_trait;

use super::{GetPermissionByUserIdError, Permission, Role, RoleError, RoleRepositoryInterface};

#[async_trait]
pub trait RoleServiceInterface {
    async fn get_list_role(&self) -> Result<Vec<Role>, RoleError>;
    async fn get_permissions_by_role_id(
        &self,
        role_id: i32,
    ) -> Result<Vec<Permission>, GetPermissionByUserIdError>;
}

pub struct RoleService {
    _role_repository: Arc<dyn RoleRepositoryInterface + Send + Sync + 'static>,
}

impl RoleService {
    pub fn new(
        _role_repository: Arc<dyn RoleRepositoryInterface + Send + Sync + 'static>,
    ) -> RoleService {
        RoleService { _role_repository }
    }
}

#[async_trait]
impl RoleServiceInterface for RoleService {
    async fn get_list_role(&self) -> Result<Vec<Role>, RoleError> {
        let roles = match self._role_repository.find().await {
            Ok(r) => r,
            Err(e) => return Err(RoleError::InternalServerError(e.to_string())),
        };

        Ok(roles)
    }

    async fn get_permissions_by_role_id(
        &self,
        role_id: i32,
    ) -> Result<Vec<Permission>, GetPermissionByUserIdError> {
        let permissions = match self
            ._role_repository
            .find_permissions_by_role_id(role_id)
            .await
        {
            Ok(p) => p,
            Err(e) => {
                return Err(GetPermissionByUserIdError::InternalServerError(
                    e.to_string(),
                ))
            }
        };

        Ok(permissions)
    }
}
