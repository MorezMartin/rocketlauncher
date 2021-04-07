use serde::{ Serialize, Deserialize };
use crate::crud::{ Crud, CrudResult, CrudError };

#[derive(Serialize, Deserialize, Clone)]
pub enum Auth {
    TypeAuth { 
        user_nids : Vec<String>,
        group_nids : Vec<String>,
        tree: String,
        auths: Authorizations
    },
    AssetAuth {
        tree: String,
        asset_nid: String,
        auths: Authorizations
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Authorizations {
    create: bool,
    read: bool,
    update: bool,
    delete: bool
}

pub enum Auths {
    AuthCreate,
    AuthUpdate,
    AuthDelete,
    AuthGet
}

impl Crud for Auth {}

