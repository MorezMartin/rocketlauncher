pub mod users;
mod groups;
mod auths;
pub fn get_routes() -> Vec<rocket::Route> {
    routes![
        users::post_user,
        users::get_user,
        users::get_users,
        users::get_token,
        groups::post_group,
        groups::get_group,
        groups::get_groups
    ]
}

