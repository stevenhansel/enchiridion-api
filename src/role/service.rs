use async_trait::async_trait;
use strum::IntoEnumIterator;

use super::{ApplicationPermission, PermissionObject, RoleObject, DEFAULT_ROLES, GetRoleByNameError};

#[async_trait]
pub trait RoleServiceInterface {
    fn list_role(&self) -> Vec<RoleObject>;
    fn get_role_by_value(&self, name: &str) -> Result<RoleObject, GetRoleByNameError>;
    fn list_permission(&self) -> Vec<PermissionObject>;
}

pub struct RoleService {}

impl RoleService {
    pub fn new() -> RoleService {
        RoleService {}
    }
}

#[async_trait]
impl RoleServiceInterface for RoleService {
    fn list_role(&self) -> Vec<RoleObject> {
        DEFAULT_ROLES
            .into_iter()
            .map(|r| RoleObject {
                name: r.name,
                value: r.value,
                description: r.description,
                permissions: r
                    .permissions
                    .into_iter()
                    .map(|p| PermissionObject {
                        label: p.label(),
                        value: p.value(),
                    })
                    .collect(),
            })
            .collect()
    }

    fn get_role_by_value(&self, val: &str) -> Result<RoleObject, GetRoleByNameError> {
        match self.list_role().into_iter().find(|r| r.value == val) {
            Some(role) => Ok(role),
            None => Err(GetRoleByNameError::RoleNotFound(
                "Unable to find role within the system",
            )),
        }
    }

    fn list_permission(&self) -> Vec<PermissionObject> {
        ApplicationPermission::iter()
            .map(|p| PermissionObject {
                label: p.label(),
                value: p.value(),
            })
            .collect()
    }
}
