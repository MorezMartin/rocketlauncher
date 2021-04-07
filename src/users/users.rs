use pwhash::bcrypt;
use sled_extensions::Db;
use serde::{ Serialize, Deserialize };
use rocket_contrib::json::Json;
use rocket::State;
use crate::crud::{Crud, CrudResult, CrudError};
use rocket::response::Debug;
use rocket::http::Cookies;
use nanoid::nanoid;
use rocket_csrf::CsrfToken;
use rocket_contrib::templates::Template;

type AuthToken = String;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Users {
    UserCreate { nickname: String, email: String, password: String },
    UserUpdate { nickname: Option<String>, name: Option<String>, surname: Option<String>, email: Option<String>, password: Option<String>, new_password: Option<String>, auth_token: AuthToken },
    UserDelete(AuthToken),
    UserLogin { email: String, password: String },
    UserLogout(AuthToken),
    UserResp { nickname: String, name: Option<String>, surname: Option<String> }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    nid: String,
    nickname: String,
    name: Option<String>,
    surname: Option<String>,
    email: String,
    password: String
}

impl Crud for User {}

impl User {
    fn resp(&self) -> Users {
        Users::UserResp {
            nickname: self.nickname.clone(),
            name: self.name.clone(),
            surname: self.surname.clone(),
        }
    }
    fn get_all(db: &State<Db>) -> CrudResult<Vec<Users>> {
        let us: Vec<Users> = User::obj_iter(&db, "user")?.values().map(|x| x.unwrap().resp()).collect();
        Ok(us)
    }
    fn get_by_email(db: &State<Db>, email: &str) -> CrudResult<User> {
        let u = User::obj_iter(&db, "user")?.values().map(|x| x.unwrap()).find(|x| x.email == email);
        match u {
            Some(user) => Ok(user),
            None => Err(Debug(CrudError::NotFoundError))
        }
    }
}

#[post("/user", format="application/json", data="<user>")]
pub fn post_user(user: Json<Users>, db: State<Db>, csrf_token: CsrfToken, mut cookies: Cookies) -> CrudResult<Json<Users>> {
    match user.into_inner() {
        Users::UserCreate { nickname: n, email: e, password: p } => {
            let unid = User::get_by_email(&db, &e);
            match unid {
                Ok(user) => Err(Debug(CrudError::ExistsError)),
                Err(x) => {
                    let nid = nanoid!();
                    let u = User { 
                        nid: nid.clone(),
                        nickname: n,
                        name: None,
                        surname: None,
                        email: e,
                        password: bcrypt::hash(p).unwrap()
                    };
                    u.clone().new(&db, "user", &nid)?;
                    Ok(Json(u.resp()))
                }
            }
        }
        Users::UserLogin { email: e, password: p } => {
            let udb = User::get_by_email(&db, &e)?;
            if bcrypt::verify(p, &udb.password) {
                Ok(Json(User::add_private(&db, cookies, "user", &udb.nid, udb.clone().nid)?.resp()))
            } else {
                Err(Debug(CrudError::ForbiddenError))
            }
        }
        Users::UserLogout(auth_token) => {
            if let Err(_) = csrf_token.verify(&auth_token) {
                return Err(Debug(CrudError::TokenError))
            }
            let cookie = cookies.get_private("user");
            match cookie {
                Some(c) => {
                    cookies.remove_private(c);
                    Ok(Json(Users::UserLogout(auth_token)))
                },
                None => return Err(Debug(CrudError::ForbiddenError))
            }
        }
        Users::UserUpdate { nickname: nn, name: n, surname: sn, email: e, password: p, new_password: np, auth_token } => {
            if let Err(_) = csrf_token.verify(&auth_token) {
                return Err(Debug(CrudError::TokenError))
            }
            let cookie = cookies.get_private("user");
            let cookie = match cookie {
                Some(c) => c,
                None => return Err(Debug(CrudError::ForbiddenError))
            };
            let udb = User::get(&db, "user", &cookie.value())?;
            let nn = match nn {
                Some(w) => w,
                None => udb.nickname
            };
            let n = match n {
                Some(w) => Some(w),
                None => udb.name
            };
            let sn = match sn {
                Some(w) => Some(w),
                None => udb.surname
            };
            let p = match p {
                Some(w) => w,
                None => "".to_string()
            };
            let e = match e {
                Some(w) => {
                    if bcrypt::verify(p.clone(), &udb.password) { 
                        w 
                    } else { 
                        return Err(Debug(CrudError::ForbiddenError)) 
                    }
                },
                None => udb.email
            };
            let np = match np {
                Some(w) => {
                    if bcrypt::verify(p.clone(), &udb.password) {
                        if !bcrypt::verify(w.clone(), &udb.password) {
                            bcrypt::hash(w).unwrap()
                        } else {
                            return Err(Debug(CrudError::ExistsError))
                        }
                    } else {
                        return Err(Debug(CrudError::ForbiddenError))
                    }
                },
                None => udb.password
            };
            let u = User {
                nid: udb.nid.clone(),
                nickname: nn,
                name: n,
                surname: sn,
                email: e,
                password: np
            };
            u.clone().update(&db, "user", &udb.nid)?;
            Ok(Json(u.resp()))
        }
        _ => Err(Debug(CrudError::NotFoundError))
    }
}

#[get("/token", format="application/json")]
pub fn get_token(db: State<Db>, csrf_token: CsrfToken, mut cookies: Cookies) -> CrudResult<Json<String>> {
    let c = cookies.get_private("user");
    let auth_token = match c {
        Some(cookie) => {
            if User::exists(&db, "user", cookie.value()) {
                csrf_token.authenticity_token()
            } else {
                "b".to_string()
            }
        }
        _ => "a".to_string()
    };
    Ok(Json(auth_token))
}

#[get("/user?<email>&<nid>", format="application/json")]
pub fn get_user(email: Option<String>, nid: Option<String>, db: State<Db>) -> CrudResult<Json<Users>> {
    match (nid, email) {
        (Some(n), _) => Ok(Json(User::get(&db, "user", &n)?.resp())),
        (_, Some(e)) => Ok(Json(User::get_by_email(&db, &e)?.resp())),
        (_, _) => Err(Debug(CrudError::NotFoundError)),
    }
}

#[get("/users", format="application/json")]
pub fn get_users(db: State<Db>) -> CrudResult<Json<Vec<Users>>> {
    let us = User::get_all(&db)?;
    println!("      👋 coucou");
    Ok(Json(us))
}
