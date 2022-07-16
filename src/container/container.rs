use shaku::module;

use crate::auth::{AuthService, AuthRepository};
use crate::building::{BuildingService, BuildingRepository};
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
            BuildingService,
            BuildingRepository,
        ],
        providers = []
    }
}
