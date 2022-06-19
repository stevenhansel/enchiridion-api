use shaku::module;

use crate::auth::service::AuthService;
use crate::user::{repository::UserRepository, service::UserService};

module! {
    pub Container {
        components = [AuthService, UserRepository, UserService],
        providers = []
    }
}
