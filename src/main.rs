use std::net::TcpListener;

use enchiridion_api::run;
use sqlx::PgPool;

struct UserRepository<'a> {
    _db: &'a PgPool,
}

impl<'a> UserRepository<'a> {
    pub fn new(db: &PgPool) -> UserRepository {
        UserRepository { _db: db }
    }

    fn get_users(&self) -> bool {
        true
    }
}

struct AuthService<'a> {
    _user_repository: &'a UserRepository<'a>,
}

impl<'a> AuthService<'a> {
    pub fn new(user_repository: &'a UserRepository<'a>) -> AuthService<'a> {
        AuthService {
            _user_repository: user_repository,
        }
    }

    fn login(self) -> bool {
        self._user_repository.get_users()
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let pool = PgPool::connect("").await.unwrap();

    let repo = UserRepository::new(&pool);
    let auth_service = AuthService::new(&repo);

    let listener = TcpListener::bind(format!("127.0.0.1:{}", 8080))
        .expect(format!("Failed to bind port {}", 8080).as_str());

    run(listener)?.await
}
