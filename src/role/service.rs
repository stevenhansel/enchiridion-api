use std::sync::Arc;

use shaku::{Component, Interface};
use async_trait::async_trait;

use super::{Role, RoleRepositoryInterface};

#[async_trait]
pub trait RoleServiceInterface: Interface {
    async fn get_list_role(&self) -> Result<Vec<Role>, String>;
}

#[derive(Component)]
#[shaku(interface = RoleServiceInterface)]
pub struct RoleService {
    #[shaku(inject)]
    _role_repository: Arc<dyn RoleRepositoryInterface>  
}

#[async_trait]
impl RoleServiceInterface for RoleService {
    async fn get_list_role(&self) -> Result<Vec<Role>, String> {
        let roles = match self._role_repository.find().await {
            Ok(r) => r,
            Err(e) => return Err(e.to_string()),
        };

        Ok(roles)
    }
}
