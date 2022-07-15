use shaku::module;

use crate::auth::service::AuthService;
use crate::building::{BuildingService, BuildingRepository};
use crate::user::{repository::UserRepository, service::UserService};
use crate::role::{RoleService, RoleRepository};

module! {
    pub Container {
        components = [
            AuthService,
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
