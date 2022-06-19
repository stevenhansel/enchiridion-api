use std::sync::Arc;

use async_trait::async_trait;
use shaku::{Component, Interface};

use super::{
    domain::User,
    repository::{InsertUserParams, UserRepositoryInterface},
};

#[async_trait]
pub trait UserServiceInterface: Interface {
    async fn create<'a>(&self, params: &'a InsertUserParams) -> Result<i32, String>;
    async fn get_user_by_id(&self, id: i32) -> Result<User, String>;
    async fn get_user_by_email(&self, email: String) -> Result<User, String>;
}

#[derive(Component)]
#[shaku(interface = UserServiceInterface)]
pub struct UserService {
    #[shaku(inject)]
    _user_repository: Arc<dyn UserRepositoryInterface>,
}

#[async_trait]
impl UserServiceInterface for UserService {
    async fn create<'a>(&self, params: &'a InsertUserParams) -> Result<i32, String> {
        let id = match self._user_repository.create(params).await {
            Ok(id) => id,
            Err(e) => return Err(e.to_string()),
        };

        Ok(id)
    }

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
