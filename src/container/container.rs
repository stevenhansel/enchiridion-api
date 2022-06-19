use shaku::module;

use crate::auth::service::AuthService;
use crate::user::repository::UserRepository;

module! {
    pub Container {
        components = [UserRepository, AuthService],
        providers = []
    }
}
