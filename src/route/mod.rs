use rocket::{Rocket, Route};

use api::*;
use files::*;
use users::*;

mod users;
mod files;

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
        .mount("/api", routes![app])
        .mount("/", routes![app, app_path])
}
