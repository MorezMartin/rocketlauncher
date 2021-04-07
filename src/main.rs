#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
extern crate nanoid;

use sled_extensions::{ Config, Db };
use rocket_contrib::helmet::SpaceHelmet;
use rocket_contrib::templates::Template;
use rocket_contrib::serve::StaticFiles;
use rocket_crud::users;
use rocket_cors::{AllowedHeaders, AllowedOrigins};
use rocket::http::Method;

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

    let allowed_origins = AllowedOrigins::some_exact(&["http://127.0.0.1:8080"]);
    let cors = rocket_cors::CorsOptions {
//        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post].into_iter().map(From::from).collect(),
//        allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors();
    let cors = match cors {
        Ok(c) => c,
        _ => return ()
    };

    rocket::ignite()
        .attach(Template::fairing())
        .attach(rocket_csrf::Fairing::default())
        .attach(SpaceHelmet::default())
        .attach(cors)
        .manage(db)
        .mount("/public", StaticFiles::from("static"))
        .mount("/", get_routes())
        .launch();
}
