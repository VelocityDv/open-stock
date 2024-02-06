use crate::catchers::Validated;
use okapi::openapi3::OpenApi;
use open_stock::{InternalDb, Session};
use rocket::{get, http::CookieJar, post, serde::json::Json};
use rocket_db_pools::Connection;
use rocket_okapi::settings::OpenApiSettings;
use rocket_okapi::{openapi, openapi_get_routes_spec};

use crate::guards::Convert;
use crate::{
    check_permissions,
    methods::{cookie_status_wrapper, Action, Error, ErrorResponse},
    pool::Db,
};

use super::Store;

pub fn documented_routes(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![
        settings: get, get_all, get_by_code, generate, update
    ]
}

#[openapi(tag = "Store")]
#[get("/")]
pub async fn get_all(db: InternalDb, session: Session) -> Convert<Vec<Store>> {
    check_permissions!(session.clone(), Action::FetchStore);
    Store::fetch_all(session, &db.0).await.into()
}

#[openapi(tag = "Store")]
#[get("/<id>")]
pub async fn get(db: InternalDb, session: Session, id: &str) -> Convert<Store> {
    check_permissions!(session.clone(), Action::FetchStore);
    Store::fetch_by_id(id, session, &db.0).await.into()
}

#[openapi(tag = "Store")]
#[get("/code/<code>")]
pub async fn get_by_code(db: InternalDb, session: Session, code: &str) -> Convert<Store> {
    check_permissions!(session.clone(), Action::FetchStore);
    Store::fetch_by_code(code, session, &db.0).await.into()
}

#[openapi(tag = "Store")]
#[post("/generate")]
async fn generate(db: InternalDb, session: Session) -> Convert<Vec<Store>> {
    check_permissions!(session.clone(), Action::GenerateTemplateContent);
    Store::generate(session, &db.0).await.into()
}

#[openapi(tag = "Store")]
#[post("/<id>", data = "<input_data>")]
async fn update(
    db: InternalDb,
    session: Session,
    input_data: Validated<Json<Store>>,
    id: &str,
) -> Result<Json<Store>, Error> {
    check_permissions!(session.clone(), Action::ModifyStore);
    Store::update(input_data.data(), session, id, &db.0)
        .await
        .into()
}
