use std::sync::Arc;

use argon2::{
    password_hash::{PasswordHash, PasswordHasher},
    Argon2,
};

pub struct RegisterParams {
    pub name: String,
    pub email: String,
    pub password: String,
    pub reason: String,
}

#[async_trait]
pub trait AuthServiceInterface: Interface {
    async fn register(&self, params: RegisterParams) -> Result<User, String>;
}

#[derive(Component)]
#[shaku(interface = AuthServiceInterface)]
pub struct AuthService {
    #[shaku(inject)]
    _user_repository: Arc<dyn UserRepositoryInterface>,
}

#[async_trait]
impl AuthServiceInterface for AuthService {
    async fn register(&self, params: RegisterParams) -> Result<User, String> {
        let hash = match Argon2::default()
            .hash_password(params.password.as_bytes(), "secretsecretsecretsecret")
        {
            Ok(p) => p.serialize(),
            Err(e) => return Err(e.to_string()),
        };
        let password = hash.as_bytes();

        let params = InsertUserParams {
            name: params.name,
            email: params.email,
            registration_reason: params.reason,
            password,
        };

        let user_id = match self._user_repository.create(&params).await {
            Ok(id) => id,
            Err(e) => return Err(e.to_string()),
        };
        let user = match self._user_repository.find_one_by_id(user_id).await {
            Ok(user) => user,
            Err(e) => return Err(e.to_string()),
        };

        Ok(user)
    }
}
