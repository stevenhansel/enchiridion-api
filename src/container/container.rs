use shaku::module;

use crate::auth::{AuthService, AuthRepository};
use crate::user::{UserRepository, UserService};

module! {
    pub Container {
        components = [
            AuthService,
            AuthRepository,
            UserRepository,
            UserService,
        ],
        providers = []
    }
}
