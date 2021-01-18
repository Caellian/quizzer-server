use std::io::Cursor;

use rocket::{Request, Response, response};
use rocket::http::ContentType;
use rocket::response::Responder;
use uuid::Uuid;
use crate::role::Role;
use crypto::bcrypt::bcrypt;
use serde_json::json;

pub static USER_COLLECTION_NAME: &'static str = "users";

static SALT: &'static [u8; 16] = include_bytes!("../password.salt");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub pw_hash: [u8; 24],
    pub user_roles: Vec<Role>,
}

impl User {
    pub fn new(username: String, password: String) -> User {
        let mut pw_hash: [u8; 24] = [0; 24];

        // TODO: Implement password pre-hashing
        bcrypt(15, SALT, password.as_bytes(), &mut pw_hash);

        let uuid = Uuid::new_v4();

        info!("Creating a new user with UUID: {}", uuid.to_string());

        User {
            id: uuid, // TODO: While highly unlikely, what if UUID exists?
            username,
            pw_hash,
            user_roles: vec![Role::Normal],
        }
    }

    pub fn response_json(&self) -> String {
        json!({
            "id": self.id.clone(),
            "username": self.username.clone(),
            "user_roles": self.user_roles,
        }).to_string()
    }
}

impl<'r> Responder<'r, 'static> for User {
    fn respond_to(self, _: &Request) -> response::Result<'static> {
        let body: String = self.response_json();

        Response::build()
            .header(ContentType::JSON)
            .sized_body(body.len(), Cursor::new(body))
            .ok()
    }
}
