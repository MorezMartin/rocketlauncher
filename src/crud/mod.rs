use sled_extensions::json::Iter;
use sled_extensions::{ Db, DbExt };
use rocket::response::Debug;
use rocket::State;
use serde::{ Serialize, Deserialize };
use rocket::http::{ Cookies, Cookie, SameSite };

pub type CrudResult<T> = Result<T, Debug<CrudError>>;

#[derive(Debug)]
pub enum CrudError {
    ExistsError,
    NotFoundError,
    ForbiddenError,
    TokenError
}

pub trait Crud {
    fn exists(db: &State<Db>, tree: &str, key: &str) -> bool {
        db.open_json_tree::<u8>(tree).unwrap().contains_key(key.as_bytes()).unwrap()
    }
    fn new(self, db: &State<Db>, tree: &str, key: &str) -> CrudResult<Self>
    where Self: for<'de> Deserialize<'de> + std::marker::Sized + Serialize + Clone + 'static {
        let tree = db.open_json_tree(tree).unwrap();
        if tree.contains_key(key.as_bytes()).unwrap() {
            Err(Debug(CrudError::ExistsError))
        } else {
            tree.compare_and_swap(key.as_bytes(), None, Some(self.clone())).unwrap();
            Ok(self)
        }
    }
    fn get(db: &State<Db>, tree: &str, key: &str) -> CrudResult<Self>
    where Self: for<'de> Deserialize<'de> + std::marker::Sized + Serialize + Clone + 'static {
        let res = db.open_json_tree(tree).unwrap().get(key.as_bytes()).unwrap();
        match res {
            Some(r) => Ok(r),
            None => Err(Debug(CrudError::NotFoundError))
        }
    }
    fn obj_iter(db: &State<Db>, tree: &str) -> CrudResult<Iter<Self>>
    where Self: for<'de> Deserialize<'de> + std::marker::Sized + Serialize + Clone + 'static {
        Ok(db.open_json_tree(tree).unwrap().iter())
    }
    fn update(self, db: &State<Db>, tree: &str, key: &str) -> CrudResult<Self>
    where Self: for<'de> Deserialize<'de> + std::marker::Sized + Serialize + Clone + 'static {
        let tree = db.open_json_tree(tree).unwrap();
        if tree.contains_key(key.as_bytes()).unwrap() {
            tree.insert(key.as_bytes(), self.clone());
            Ok(self)
        } else {
            Err(Debug(CrudError::NotFoundError))
        }
    }
    fn delete(self, db: &State<Db>, tree: &str, key: &str) -> CrudResult<Self>
    where Self: for<'de> Deserialize<'de> + std::marker::Sized + Serialize + Clone + 'static {
        let tree = db.open_json_tree(tree).unwrap();
        if tree.contains_key(key.as_bytes()).unwrap() {
            tree.compare_and_swap(key.as_bytes(), Some(self.clone()), None);
            Ok(self)
        } else {
            Err(Debug(CrudError::NotFoundError))
        }
    }
    fn add_private(db: &State<Db>, mut cookies: Cookies, tree: &'static str, key: &str, cookiebuilder: String) 
    -> CrudResult<Self>
    where Self: for<'de> Deserialize<'de> + std::marker::Sized + Serialize + Clone + 'static {
        let tdb = db.open_json_tree::<u8>(tree).unwrap();
        if tdb.contains_key(key.as_bytes()).unwrap() {
            let cookie = Cookie::build(tree, cookiebuilder)
                .secure(true)
                .http_only(true)
                .same_site(SameSite::Lax)
                .finish();
            cookies.add_private(cookie);
            let res = db.open_json_tree(tree).unwrap().get(key.as_bytes()).unwrap();
            Ok(res.unwrap())
        } else {
            Err(Debug(CrudError::NotFoundError))
        }
    }
}
