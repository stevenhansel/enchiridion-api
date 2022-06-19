pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password: Vec<u8>,
    pub registration_reason: String,
}
