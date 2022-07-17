use std::sync::Arc;

use async_trait::async_trait;

use super::{domain::User, repository::UserRepositoryInterface};

#[async_trait]
pub trait UserServiceInterface {
    async fn get_user_by_id(&self, id: i32) -> Result<User, String>;
    async fn get_user_by_email(&self, email: String) -> Result<User, String>;
}

pub struct UserService {
    _user_repository: Arc<dyn UserRepositoryInterface + Send + Sync + 'static>,
}

impl UserService {
    pub fn new(
        _user_repository: Arc<dyn UserRepositoryInterface + Send + Sync + 'static>,
    ) -> UserService {
        UserService { _user_repository }
    }
}

#[async_trait]
impl UserServiceInterface for UserService {
    async fn get_user_by_id(&self, id: i32) -> Result<User, String> {
        let user = match self._user_repository.find_one_by_id(id).await {
            Ok(u) => u,
            Err(e) => return Err(e.to_string()),
        };

        Ok(user)
    }

    async fn get_user_by_email(&self, email: String) -> Result<User, String> {
        let user = match self._user_repository.find_one_by_email(email).await {
            Ok(u) => u,
            Err(e) => return Err(e.to_string()),
        };

        Ok(user)
    }
}
