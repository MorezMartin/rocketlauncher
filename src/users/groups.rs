use rocket::State;
use rocket_contrib::json::Json;
use crate::crud::{Crud, CrudResult, CrudError};
use sled_extensions::Db;
use serde::{ Serialize, Deserialize };
use rocket::http::Cookies;
use rocket::response::Debug;
use nanoid::nanoid;


#[derive(Serialize, Deserialize, Clone)]
pub enum Groups {
    GroupCreate { name: String, description: Option<String> },
    GroupDelete,
    GroupResp { name: String, description: Option<String>, members: Vec<String> },
    AddUser { group_nid: String, user_nid: String }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Group {
    nid: String,
    user_nid: String,
    members: Vec<String>,
    name: String,
    description: Option<String>,
}

impl Crud for Group {}

impl Group {
    fn is_member(&self, nid: String) -> bool {
        let i = self.members.iter().find(|&x| x==&nid);
        match i {
            Some(i) => true,
            None => false
        }
    }
    fn add_user(&self, db: &State<Db>, nid: String) -> CrudResult<Groups> {
        if !self.is_member(nid.clone()) {
            let mut s = self.clone();
            s.members.push(nid.clone());
            Ok(s.update(&db, "group", &nid)?.resp())
        } else {
            Err(Debug(CrudError::ExistsError))
        }
    }
    fn resp(&self) -> Groups {
        let g = Groups::GroupResp { name: self.name.clone(), description: self.description.clone(), members: self.members.clone() };
        g
    }
    fn get_all(db: &State<Db>) -> CrudResult<Vec<Groups>> {
        let gs: Vec<Groups> = Group::obj_iter(&db, "group")?.values().map(|x| x.unwrap().resp()).collect();
        Ok(gs)
    }
}

#[post("/group", format="application/json", data="<group>")]
pub fn post_group(group: Json<Groups>, db: State<Db>, mut cookies: Cookies)
-> CrudResult<Json<Groups>> {
    match group.into_inner() {
        Groups::GroupCreate { name, description } => { 
            let nid = nanoid!();
            let user_nid = match cookies.get_private("user") {
                Some(c) => c.value().to_string(),
                None => return Err(Debug(CrudError::ForbiddenError))
            };
            let members = vec![user_nid.clone()];
            let g = Group {
                nid: nid.clone(),
                user_nid: user_nid,
                members: members.to_vec(),
                name: name,
                description: description
            };
            Ok(Json(g.new(&db, "group", &nid)?.resp()))
        }
        Groups::AddUser { group_nid, user_nid } => {
            let mut g = Group::get(&db, "group", &group_nid)?;
            Ok(Json(g.add_user(&db, user_nid)?))
        }
        _ => { Err(Debug(CrudError::NotFoundError)) }
    }
}

#[get("/group?<nid>", format="application/json")]
pub fn get_group(nid: String, db: State<Db>, mut cookies: Cookies) -> CrudResult<Json<Groups>> {
    Ok(Json(Group::get(&db, "group", &nid)?.resp()))
}

#[get("/groups", format="application/json")]
pub fn get_groups(db: State<Db>) -> CrudResult<Json<Vec<Groups>>> {
    let gs = Group::get_all(&db)?;
    println!("      👋 coucou");
    Ok(Json(gs))
}

