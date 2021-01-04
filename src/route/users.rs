use bson::{Bson, doc, Document, from_bson, from_document};
use bson::spec::BinarySubtype;
use mongodb::Database;
use rocket::{State};
use rocket::http::{CookieJar, Status};
use rocket::request::{Form, FromForm};
use uuid::Uuid;

use crate::jwt::UserRolesToken;
use crate::user::{User, USER_COLLECTION_NAME};
use crate::role::Role;
use crate::error::{Problem, problems};
use crate::config::Config;

/* TODO: Support paging
// Responder isn't implemented for Vec.
#[get("/")]
pub async fn user_list(db: State<'_, Database>) -> Result<Vec<User>, Problem> {
    let mut user_cursor = db.collection(USER_COLLECTION_NAME)
        .find(None, None)
        .await
        .map_err(|e| Problem::from(e))?;

    let mut users: Vec<User> = vec![];
    while let Some(user_result) = user_cursor.next().await {
        let user_document = Bson::Document(user_result.unwrap());
        match from_bson(user_document) {
            Ok(user) => {
                users.push(user)
            }
            Err(_) => {
                // show must go on?
                warn!("Unable to deserialize User document.")
            }
        }
    }

    Ok(users)
}
*/

#[inline]
pub fn parse_uuid<'r>(id: &String) -> Result<Uuid, Problem> {
    match Uuid::parse_str(id.clone().as_str()) {
        Ok(it) => Ok(it),
        Err(_) => Err(
            problems::parse_problem()
                .insert_serialized("parsed", id.clone())
                .detail("UUID parsing failed.")
                .clone()
        )
    }
}

#[inline]
pub fn filter_user_id(id: Uuid) -> Document {
    doc! {
        "id": Bson::Binary(bson::Binary {
            subtype: BinarySubtype::Uuid,
            bytes: id.as_bytes().to_vec(),
        })
    }
}

#[inline]
pub fn filter_user_username(username: String) -> Document {
    doc! {
        "username": username
    }
}

#[get("/<id>")]
pub async fn user_get(id: String, db: State<'_, Database>) -> Result<Option<User>, Problem> {
    let uuid = parse_uuid(&id)?;

    let user_document = db.collection(USER_COLLECTION_NAME).find_one(
        filter_user_id(uuid),
        None,
    ).await
        .map_err(|e| Problem::from(e))?;

    match user_document {
        Some(doc) => Ok(Some(
            from_bson(Bson::Document(doc))
                .map_err(|e| Problem::from(e))?
        )),
        None => Ok(None)
    }
}

#[derive(Clone, FromForm)]
pub struct CreateUser {
    username: String,
    password: String,
}

#[inline]
fn bad_username_problem<Username: Into<String>, Detail: Into<String>>(username: Username, detail: Detail) -> Problem {
    Problem::new_untyped(
        Status::BadRequest,
        "Bad username."
    )
        .insert_serialized("username", username.into())
        .detail(detail)
        .clone()
}

#[inline]
fn bad_password_problem<S: Into<String>>(detail: S) -> Problem {
    Problem::new_untyped(
        Status::BadRequest,
        "Bad password."
    )
        .detail(detail)
        .clone()
}

#[inline]
fn user_not_found(id: Uuid) -> Problem {
    Problem::new_untyped(
        Status::NotFound,
        "User doesn't exist."
    )
        .insert_serialized("id", id.to_string())
        .clone()
}

impl CreateUser {
    pub fn validate(&self) -> Result<(), Problem> {

        if self.username.len() < 5 {
            return Err(
                bad_username_problem(
                    self.username.clone(),
                    "Username must be at least 5 characters (bytes) long."
                )
            )
        }

        if self.username.len() > 32 {
            return Err(
                bad_username_problem(
                    self.username.clone(),
                    "Username can't be longer than 32 (bytes) characters."
                )
            )
        }

        if self.password.len() < 8 {
            return Err(
                bad_password_problem("Password must be at least 8 characters (bytes) long.")
            )
        }

        if self.password.len() > 50 {
            return Err(
                bad_password_problem("Passwords longer than 50 characters (bytes) can't be hashed properly.")
            )
        }

        Ok(())
    }
}

#[post("/", data = "<create_user>")]
pub async fn user_create<'a>(create_user: Form<CreateUser>, cookies: &'a CookieJar<'_>, db: State<'_, Database>, c: State<'_, Config>) -> Result<User, Problem> {
    create_user.validate()?;

    let mut user = User::new(
        create_user.username.clone(),
        create_user.password.clone(),
    );

    if c.admin_usernames.contains(&user.username) {
        user.user_roles.push(Role::Admin);
    }

    if db.collection(USER_COLLECTION_NAME).find_one(
        filter_user_username(user.username.clone()),
        None,
    ).await.expect("Unable to query by username.").is_some() {
        return Err(
            bad_username_problem(
                user.username.clone(),
                "User with that username already exists."
            )
        );
    }

    db.collection(USER_COLLECTION_NAME)
        .insert_one(bson::to_document(&user)
                        .expect("Unable to serialize User struct into BSON."),
                    None).await
        .map_err(|e| Problem::from(e))?;

    let urt = UserRolesToken::new(&user.clone());
    cookies.add(urt.cookie()?);

    Ok(user)
}

#[delete("/<id>")]
pub async fn user_delete(id: String, db: State<'_, Database>) -> Result<User, Problem> {
    let uuid = parse_uuid(&id)?;

    let removed_document = db.collection(USER_COLLECTION_NAME).find_one_and_delete(
        filter_user_id(uuid),
        None,
    ).await
        .map_err(|e| Problem::from(e))?;


    match removed_document {
        Some(user_document) => {
            let user: User = from_document(user_document)
                .expect("Unable to deserialize User struct from BSON.");

            Ok(user)
        }
        None => {
            Err(
                user_not_found(uuid)
            )
        }
    }
}
