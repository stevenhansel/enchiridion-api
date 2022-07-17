use std::sync::Arc;

use async_trait::async_trait;

use super::{Role, RoleError, RoleRepositoryInterface};

#[async_trait]
pub trait RoleServiceInterface {
    async fn get_list_role(&self) -> Result<Vec<Role>, RoleError>;
}

pub struct RoleService {
    _role_repository: Arc<dyn RoleRepositoryInterface + Send + Sync + 'static>  
}

impl RoleService {
    pub fn new(_role_repository: Arc<dyn RoleRepositoryInterface + Send + Sync + 'static>) -> RoleService {
        RoleService {
            _role_repository,
        }
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
}
