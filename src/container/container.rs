use shaku::module;

use crate::auth::{AuthService, AuthRepository};
use crate::user::{UserRepository, UserService};
use crate::role::{RoleService, RoleRepository};

module! {
    pub Container {
        components = [
            AuthService,
            AuthRepository,
            UserRepository,
            UserService,
            RoleService,
            RoleRepository,
        ],
        providers = []
    }
}
