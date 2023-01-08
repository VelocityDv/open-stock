use rocket::http::CookieJar;
use rocket::{http::Status, get, patch};
use rocket::{routes, post};
use rocket::serde::json::Json;
use sea_orm_rocket::{Connection};
use crate::check_permissions;
use crate::methods::cookie_status_wrapper;
use crate::pool::Db;
use crate::methods::employee::Action;

use super::{Supplier, SupplierInput};

pub fn routes() -> Vec<rocket::Route> {
    routes![get, get_by_name, get_by_phone, get_by_addr, create, update, generate]
}

#[get("/<id>")]
pub async fn get(conn: Connection<'_, Db>, id: &str, cookies: &CookieJar<'_>) -> Result<Json<Supplier>, Status> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchSupplier);

    let customer = Supplier::fetch_by_id(&id, db).await.unwrap();
    Ok(Json(customer))
}

#[get("/name/<name>")]
pub async fn get_by_name(conn: Connection<'_, Db>, name: &str, cookies: &CookieJar<'_>) -> Result<Json<Vec<Supplier>>, Status> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchSupplier);

    let employee = Supplier::fetch_by_name(name, db).await.unwrap();
    Ok(Json(employee))
}

#[get("/phone/<phone>")]
pub async fn get_by_phone(conn: Connection<'_, Db>, phone: &str, cookies: &CookieJar<'_>) -> Result<Json<Vec<Supplier>>, Status> {
    let db = conn.into_inner();
    let new_phone = phone.clone();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchSupplier);

    let employee = Supplier::fetch_by_phone(new_phone, db).await.unwrap();
    Ok(Json(employee))
}

#[get("/addr/<addr>")]
pub async fn get_by_addr(conn: Connection<'_, Db>, addr: &str, cookies: &CookieJar<'_>) -> Result<Json<Vec<Supplier>>, Status> {
    let db = conn.into_inner();
    let new_addr = addr.clone();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchSupplier);

    let employee = Supplier::fetch_by_addr(new_addr, db).await.unwrap();
    Ok(Json(employee))
}

#[patch("/generate")]
async fn generate(
    conn: Connection<'_, Db>, 
    cookies: &CookieJar<'_>
) -> Result<Json<Supplier>, Status> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::GenerateTemplateContent);

    match Supplier::generate(db).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::BadRequest)
    }
}

#[post("/<id>", data = "<input_data>")]
async fn update(
    conn: Connection<'_, Db>,
    id: &str,
    cookies: &CookieJar<'_>,
    input_data: Json<SupplierInput>,
) -> Result<Json<Supplier>, Status> {
    let input_data = input_data.clone().into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::ModifySupplier);

    match Supplier::update(input_data, id, db).await {
        Ok(res) => {
            Ok(Json(res))
        },
        Err(_) => Err(Status::BadRequest),
    }
}

#[post("/", data = "<input_data>")]
pub async fn create(conn: Connection<'_, Db>, cookies: &CookieJar<'_>, input_data: Json<SupplierInput>) -> Result<Json<Supplier>, Status> {
    let new_data = input_data.clone().into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::ModifySupplier);

    match Supplier::insert(new_data, db).await {
        Ok(data) => {
            match Supplier::fetch_by_id(&data.last_insert_id, db).await {
                Ok(res) => {
                    Ok(Json(res))
                },  
                Err(reason) => {
                    println!("[dberr]: {}", reason);
                    Err(Status::InternalServerError)
                }
            }
        },
        Err(reason) => {
            println!("[dberr]: {}", reason);
            Err(Status::InternalServerError)
        },
    }
}