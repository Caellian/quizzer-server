#![feature(proc_macro_hygiene, decl_macro)]

extern crate log;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde;

use mongodb::Client;
use rocket_contrib::helmet::SpaceHelmet;

use crate::config::Config;
use crate::route::mount_routes;

mod data;
mod jwt;
mod user;
mod route;
mod config;
mod role;
mod error;

#[rocket::main]
async fn main() {
    // TODO: Deal with logging later.
    //log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    info!("Reading .env file...");
    if dotenv::dotenv().is_err() {
        error!("Unable to load .env file.")
    }

    info!("Initializing configuration...");
    let c = Config::init();

    info!("Connecting to MongoDB: {}", c.mongodb_uri);
    let client = Client::with_uri_str(c.mongodb_uri.as_str()).await
        .expect("Unable to init MongoDB client! Is URI valid?");

    info!("Using MongoDB database: {}", c.mongodb_db);
    let db = client.database(c.mongodb_db.as_str());

    info!("Igniting Rocket...");
    let mut r = rocket::ignite()
        .manage(c)
        .manage(db);

    r = mount_routes(r);

    let helmet = SpaceHelmet::default();
    r = r.attach(helmet);

    match r.launch().await {
        Ok(_) => {},
        Err(e) => {
            println!("Time to invest in a new rocket: {}", e);
        }
    };
}
