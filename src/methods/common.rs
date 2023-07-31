use std::fmt::Display;

use super::{Employee as EmployeeObj, ProductExchange};
use crate::entities::employee::Entity as Employee;
use crate::entities::session::Entity as SessionEntity;
use crate::{
    entities,
    methods::{stml::Order, Access, Action, Attendance, EmployeeAuth},
};
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use rocket::{http::CookieJar, serde::json::Json, Responder};
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QuerySelect};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Name {
    pub first: String,
    pub middle: String,
    pub last: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ContactInformation {
    pub name: String,
    pub mobile: MobileNumber,
    pub email: Email,
    pub landline: String,
    pub address: Address,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MobileNumber {
    pub number: String,
    pub valid: bool,
}

/// Performs
/// ```regex
/// ^(\+\d{1,2}\s?)?1?\-?\.?\s?\(?\d{3}\)?[\s.-]?\d{3}[\s.-]?\d{4}$
/// ```
pub fn verify_phone_number_with_country_code(ph: &str) -> bool {
    // prevent re-compilation of regex
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"^(\+\d{1,2}\s?)?1?\-?\.?\s?\(?\d{3}\)?[\s.-]?\d{3}[\s.-]?\d{4}$").unwrap();
    }

    RE.is_match(ph)
}

/// Performs:
/// ```regex
/// ^1?\-?\.?\s?\(?\d{3}\)?[\s.-]?\d{3}[\s.-]?\d{4}$
/// ```
pub fn verify_phone_number_without_country_code(ph: &str) -> bool {
    // prevent re-compilation of regex
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"^1?\-?\.?\s?\(?\d{3}\)?[\s.-]?\d{3}[\s.-]?\d{4}$").unwrap();
    }

    RE.is_match(ph)
}

impl MobileNumber {
    pub fn from(number: String) -> Self {
        let valid = if number.starts_with('+') {
            verify_phone_number_with_country_code(number.as_str())
        } else {
            verify_phone_number_without_country_code(number.as_str())
        };

        MobileNumber { number, valid }
    }
}

pub type OrderList = Vec<Order>;
pub type NoteList = Vec<Note>;
pub type HistoryList = Vec<History<ProductExchange>>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct History<T> {
    pub item: T,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Email {
    pub root: String,
    pub domain: String,
    pub full: String,
}

impl Email {
    pub fn from(email: String) -> Self {
        let split = email.split('@');
        let col = split.collect::<Vec<&str>>();

        let root = match col.first() {
            Some(root) => *root,
            None => "",
        };

        let domain = match col.get(1) {
            Some(domain) => *domain,
            None => "",
        };

        Email {
            root: root.into(),
            domain: domain.into(),
            full: email,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub message: String,
    pub author: String,
    pub timestamp: DateTime<Utc>,
}

impl Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}",
            self.timestamp.format("%d/%m/%Y %H:%M"),
            self.message
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Address {
    pub street: String,
    pub street2: String,
    pub city: String,
    pub country: String,
    pub po_code: String,
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub store_code: String,
    pub store_id: String,

    // Address is stored in the contact information.
    pub contact: ContactInformation,
}

pub type Url = String;

pub type TagList = Vec<Tag>;
pub type Tag = String;
pub type Id = String;

#[derive(Debug)]
pub struct SessionRaw {
    pub id: String,
    pub key: String,
    pub employee_id: String,
    pub expiry: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub key: String,
    pub employee: EmployeeObj,
    pub expiry: DateTime<Utc>,
}

impl Session {
    pub fn has_permission(self, permission: Action) -> bool {
        let action = match self
            .employee
            .level
            .into_iter()
            .find(|x| x.action == permission)
        {
            Some(e) => e,
            None => Access {
                action: permission,
                authority: 0,
            },
        };

        if action.action == Action::GenerateTemplateContent {
            true
        } else {
            action.authority >= 1
        }
    }
}

pub fn get_key_cookie(cookies: &CookieJar<'_>) -> Option<String> {
    cookies.get("key").map(|crumb| crumb.value().to_string())
}

pub async fn verify_cookie(key: String, db: &DatabaseConnection) -> Result<Session, DbErr> {
    let session = SessionEntity::find()
        .having(entities::session::Column::Key.eq(key.clone()))
        .find_also_related(Employee)
        .one(db)
        .await?;

    match session {
        Some((val, Some(e))) => Ok(Session {
            id: val.id,
            key: val.key,
            employee: EmployeeObj {
                id: e.id.clone(),
                rid: e.rid.clone(),
                name: serde_json::from_value::<Name>(e.name.clone()).unwrap(),
                auth: serde_json::from_value::<EmployeeAuth>(e.auth.clone()).unwrap(),
                contact: serde_json::from_value::<ContactInformation>(e.contact.clone()).unwrap(),
                clock_history: serde_json::from_value::<Vec<History<Attendance>>>(
                    e.clock_history.clone(),
                )
                .unwrap(),
                level: serde_json::from_value::<Vec<Access<Action>>>(e.level).unwrap(),
            },
            expiry: DateTime::from_utc(val.expiry, Utc),
        }),
        Some((_, None)) | None => Err(DbErr::RecordNotFound(format!(
            "Record {} does not exist.",
            key
        ))),
    }
}

pub async fn _handle_cookie(
    db: &DatabaseConnection,
    cookies: &CookieJar<'_>,
) -> Result<Session, DbErr> {
    match get_key_cookie(cookies) {
        Some(val) => verify_cookie(val, db).await,
        None => Err(DbErr::Custom("Cookies not set.".to_string())),
    }
}

pub async fn cookie_status_wrapper(
    db: &DatabaseConnection,
    cookies: &CookieJar<'_>,
) -> Result<Session, Error> {
    match get_key_cookie(cookies) {
        Some(val) => match verify_cookie(val, db).await {
            Ok(v) => Ok(v),
            Err(err) => {
                println!("[err]: {}", err);
                Err(ErrorResponse::custom_unauthorized(
                    "Unable to validate cookie, user does not have valid session.",
                ))
            }
        },
        None => Err(ErrorResponse::custom_unauthorized(
            "Unable to fetch user cookie.",
        )),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    message: String,
}

impl ErrorResponse {
    pub fn create_error(message: &str) -> Error {
        Error::StandardError(Json(ErrorResponse {
            message: message.to_string(),
        }))
    }

    pub fn input_error() -> Error {
        Error::InputError(Json(ErrorResponse {
            message: "Unable to update fields due to malformed inputs".to_string(),
        }))
    }

    pub fn unauthorized(action: Action) -> Error {
        Error::Unauthorized(Json(ErrorResponse {
            message: format!("User lacks {:?} permission.", action),
        }))
    }

    pub fn custom_unauthorized(message: &str) -> Error {
        Error::Unauthorized(Json(ErrorResponse {
            message: message.to_string(),
        }))
    }

    pub fn db_err(message: sea_orm::DbErr) -> Error {
        Error::DbError(Json(ErrorResponse {
            message: format!("SQL error, reason: {}", message),
        }))
    }
}

#[derive(Debug, Responder)]
pub enum Error {
    #[response(status = 500, content_type = "json")]
    StandardError(Json<ErrorResponse>),
    #[response(status = 400, content_type = "json")]
    InputError(Json<ErrorResponse>),
    #[response(status = 401, content_type = "json")]
    Unauthorized(Json<ErrorResponse>),
    #[response(status = 500, content_type = "json")]
    DbError(Json<ErrorResponse>),
    #[response(status = 500, content_type = "text")]
    DemoDisabled(String),
}
