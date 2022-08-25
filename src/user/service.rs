use std::sync::Arc;

use async_trait::async_trait;

use crate::database::PaginationResult;

use super::{
    domain::UserDetail, repository::UserRepositoryInterface, FindUserParams, ListUserError,
    UpdateUserApprovalError, User, UserStatus,
};

pub struct ListUserParams {
    pub page: i32,
    pub limit: i32,
    pub query: Option<String>,
    pub status: Option<UserStatus>,
    pub role_id: Option<i32>,
}

#[async_trait]
pub trait UserServiceInterface {
    async fn list_user(
        &self,
        params: ListUserParams,
    ) -> Result<PaginationResult<User>, ListUserError>;
    async fn get_user_by_id(&self, id: i32) -> Result<UserDetail, String>;
    async fn get_user_by_email(&self, email: String) -> Result<UserDetail, String>;
    async fn update_user_approval(
        &self,
        user_id: i32,
        approve: bool,
    ) -> Result<(), UpdateUserApprovalError>;
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
    async fn list_user(
        &self,
        params: ListUserParams,
    ) -> Result<PaginationResult<User>, ListUserError> {
        match self
            ._user_repository
            .find(FindUserParams {
                page: params.page,
                limit: params.limit,
                query: params.query.clone(),
                status: params.status.clone(),
                role_id: params.role_id.clone(),
            })
            .await
        {
            Ok(result) => Ok(result),
            Err(e) => match e {
                _ => Err(ListUserError::InternalServerError),
            },
        }
    }

    async fn get_user_by_id(&self, id: i32) -> Result<UserDetail, String> {
        let user = match self._user_repository.find_one_by_id(id).await {
            Ok(u) => u,
            Err(e) => return Err(e.to_string()),
        };

        Ok(user)
    }

    async fn get_user_by_email(&self, email: String) -> Result<UserDetail, String> {
        let user = match self._user_repository.find_one_by_email(email).await {
            Ok(u) => u,
            Err(e) => return Err(e.to_string()),
        };

        Ok(user)
    }

    async fn update_user_approval(
        &self,
        user_id: i32,
        approve: bool,
    ) -> Result<(), UpdateUserApprovalError> {
        let user = match self._user_repository.find_one_by_id(user_id).await {
            Ok(user) => user,
            Err(e) => match e {
                sqlx::Error::RowNotFound => {
                    return Err(UpdateUserApprovalError::UserNotFound(
                        "Unable to find user in the system.",
                    ))
                }
                _ => return Err(UpdateUserApprovalError::InternalServerError),
            },
        };

        if !user.is_email_confirmed {
            return Err(UpdateUserApprovalError::UserNotConfirmed(
                "User email is not confirmed, cannot approve user before email is confirmed.",
            ));
        }

        if user.status != UserStatus::WaitingForApproval {
            return Err(UpdateUserApprovalError::UserStatusConflict(
                "User status invalid, status of the user must be waiting for approval.",
            ));
        }

        if let Err(_) = self
            ._user_repository
            .update_user_approval(user_id, approve)
            .await
        {
            return Err(UpdateUserApprovalError::InternalServerError);
        }

        Ok(())
    }
}
