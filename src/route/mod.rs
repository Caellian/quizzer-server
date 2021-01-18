use rocket::{Rocket, Route};

mod users;
mod files;
mod quiz;

use users::*;
use files::*;
use quiz::*;
use crate::error::{Problem, problems};
use uuid::Uuid;

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

pub fn user_routes() -> Vec<Route> {
    routes![
        // user_list, // Requires paging
        user_get,
        user_create,
        user_delete,
    ]
}

pub fn mount_routes(rocket: Rocket) -> Rocket {
    rocket
        .mount("/user", user_routes())
        .mount("/login", routes![app, login_submit])
        .mount("/api", routes![app])
        .mount("/quiz", routes![quiz_create, quiz_info, quiz_delete])
        .mount("/", routes![app, app_path])
}
