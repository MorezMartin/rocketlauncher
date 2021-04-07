#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
extern crate nanoid;

use sled_extensions::{ Config, Db };
use rocket_contrib::helmet::SpaceHelmet;

mod users;
mod crud;

fn get_routes() -> Vec<rocket::Route> {
    let mut r = routes![];
    r.append(&mut users::get_routes());
    r
}


fn main() {
    let db: Db = Config::default()
        .path("db")
        .open()
        .expect("Failed to open sled db");

    rocket::ignite()
        .attach(rocket_csrf::Fairing::default())
        .attach(SpaceHelmet::default())
        .manage(db)
        .mount("/", get_routes())
        .launch();
}
