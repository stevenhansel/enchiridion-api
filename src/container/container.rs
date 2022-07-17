use shaku::module;

use crate::auth::{AuthService, AuthRepository};
use crate::building::{BuildingService, BuildingRepository};
use crate::user::{UserRepository, UserService};

module! {
    pub Container {
        components = [
            AuthService,
            AuthRepository,
            UserRepository,
            UserService,
            BuildingService,
            BuildingRepository,
        ],
        providers = []
    }
}
