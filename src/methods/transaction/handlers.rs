use rocket::get;
use rocket::http::CookieJar;
use rocket::serde::json::Json;
use rocket::{post, routes};
use rocket_db_pools::Connection;

use super::{Transaction, TransactionInit, TransactionInput};
use crate::methods::employee::Action;
use crate::methods::{cookie_status_wrapper, Error, ErrorResponse, QuantityAlterationIntent};
use crate::pool::Db;
use crate::{apply_discount, check_permissions, Order, OrderStatus, PickStatus};

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get,
        get_by_name,
        get_all_saved,
        get_by_product_sku,
        create,
        update,
        generate,
        delete,
        deliverables_search,
        update_product_status,
        update_order_status
    ]
}

#[get("/<id>")]
pub async fn get(
    conn: Connection<Db>,
    id: String,
    cookies: &CookieJar<'_>,
) -> Result<Json<Transaction>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchTransaction);

    match Transaction::fetch_by_id(&id, session, db).await {
        Ok(transaction) => Ok(Json(transaction)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[get("/saved")]
pub async fn get_all_saved(
    conn: Connection<Db>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Transaction>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchTransaction);

    match Transaction::fetch_all_saved(session, db).await {
        Ok(transaction) => Ok(Json(transaction)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[get("/ref/<name>")]
pub async fn get_by_name(
    conn: Connection<Db>,
    name: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Transaction>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchTransaction);

    match Transaction::fetch_by_ref(name, session, db).await {
        Ok(transaction) => Ok(Json(transaction)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[get("/product/<sku>")]
pub async fn get_by_product_sku(
    conn: Connection<Db>,
    sku: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Transaction>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchTransaction);

    match Transaction::fetch_by_ref(sku, session, db).await {
        Ok(transaction) => Ok(Json(transaction)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[get("/deliverables/<store_id>")]
pub async fn deliverables_search(
    conn: Connection<Db>,
    store_id: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Order>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchTransaction);

    match Transaction::fetch_deliverable_jobs(store_id, session, db).await {
        Ok(transaction) => Ok(Json(transaction)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[get("/receivables/<store_id>")]
pub async fn receivables_search(
    conn: Connection<Db>,
    store_id: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Order>>, Error> {
    let db = conn.into_inner();
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchTransaction);

    match Transaction::fetch_receivable_jobs(store_id, session, db).await {
        Ok(transaction) => Ok(Json(transaction)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[post("/<id>", data = "<input_data>")]
async fn update(
    conn: Connection<Db>,
    id: &str,
    input_data: Json<TransactionInput>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Transaction>, Error> {
    let input_data = input_data.clone().into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::ModifyTransaction);

    match Transaction::update(input_data, session, id, db).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(ErrorResponse::input_error()),
    }
}

#[post("/status/order/<refer>", data = "<status>")]
async fn update_order_status(
    conn: Connection<Db>,
    refer: &str,
    status: Json<OrderStatus>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Transaction>, Error> {
    let status = status.clone().into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::ModifyTransaction);

    let tsn = Transaction::fetch_by_ref(refer, session.clone(), db)
        .await
        .unwrap();
    let id = tsn.get(0).unwrap().id.as_str();

    match Transaction::update_order_status(id, refer, status, session, db).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(ErrorResponse::input_error()),
    }
}

#[post("/status/product/<refer>/<pid>/<iid>", data = "<status>")]
async fn update_product_status(
    conn: Connection<Db>,
    refer: &str,
    pid: &str,
    iid: &str,
    status: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Transaction>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::ModifyTransaction);

    let tsn = Transaction::fetch_by_ref(refer, session.clone(), db)
        .await
        .unwrap();
    let id = tsn.get(0).unwrap().id.as_str();

    let product_status: PickStatus = match status {
        "picked" => PickStatus::Picked,
        "pending" => PickStatus::Pending,
        "failed" => PickStatus::Failed,
        "uncertain" => PickStatus::Uncertain,
        "processing" => PickStatus::Processing,
        _ => return Err(ErrorResponse::input_error()),
    };

    match Transaction::update_product_status(id, refer, pid, iid, product_status, session, db).await
    {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(ErrorResponse::input_error()),
    }
}

#[post("/generate/<customer_id>")]
async fn generate(
    conn: Connection<Db>,
    customer_id: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Transaction>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::GenerateTemplateContent);

    match Transaction::generate(db, customer_id, session).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(ErrorResponse::input_error()),
    }
}

#[post("/", data = "<input_data>")]
pub async fn create(
    conn: Connection<Db>,
    input_data: Json<TransactionInit>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Transaction>, Error> {
    let new_transaction = input_data.clone().into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::CreateTransaction);

    let mut quantity_alteration_intents: Vec<QuantityAlterationIntent> = vec![];

    // Make and modify the required changes to stock levels
    new_transaction.products.iter().for_each(|order| {
        order.products.iter().for_each(|product| {
            quantity_alteration_intents.push(QuantityAlterationIntent {
                variant_code: product.clone().product_code,
                product_sku: product.clone().product_sku,
                transaction_store_code: order.clone().origin.store_code,
                transaction_store_id: order.clone().origin.store_id,
                transaction_type: new_transaction.clone().transaction_type,
                quantity_to_transact: product.clone().quantity,
            });
        });
    });

    let total_paid = new_transaction
        .payment
        .iter()
        .map(|payment| payment.amount.quantity)
        .sum::<f32>();
    let total_cost = new_transaction
        .products
        .iter()
        .map(|order| {
            apply_discount(
                order.discount.clone(),
                order
                    .products
                    .iter()
                    .map(|product| {
                        apply_discount(
                            product.discount.clone(),
                            product.product_cost * product.quantity,
                        )
                    })
                    .sum::<f32>(),
            )
        })
        .sum::<f32>();

    if (total_paid - total_cost).abs() > 0.1 {
        return Err(ErrorResponse::create_error(
            "Payment amount does not match product costs.",
        ));
    }

    match Transaction::insert(new_transaction, session.clone(), db).await {
        Ok(data) => {
            Transaction::process_intents(session.clone(), db, quantity_alteration_intents).await;

            match Transaction::fetch_by_id(&data.last_insert_id, session, db).await {
                Ok(res) => Ok(Json(res)),
                Err(reason) => Err(ErrorResponse::db_err(reason)),
            }
        }
        Err(_) => Err(ErrorResponse::input_error()),
    }
}

#[post("/delete/<id>")]
async fn delete(conn: Connection<Db>, id: &str, cookies: &CookieJar<'_>) -> Result<(), Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::DeleteTransaction);

    match Transaction::delete(id, session, db).await {
        Ok(_res) => Ok(()),
        Err(_) => Err(ErrorResponse::input_error()),
    }
}
