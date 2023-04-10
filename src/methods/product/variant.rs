use std::fmt::Display;
use chrono::{DateTime, Utc, Days};
use sea_orm::{DbConn, DbErr, EntityTrait, QuerySelect, ColumnTrait, Set, ActiveModelTrait, InsertResult};
use serde::{Deserialize, Serialize};

use serde_json::json;
use uuid::Uuid;
use crate::entities::promotion;
use crate::methods::{StockList, HistoryList, Url, DiscountValue, Id};
use crate::entities::prelude::Promotion as Promotions;

pub type VariantIdTag = Vec<VariantId>;
type VariantId = String;

pub type VariantCategoryList = Vec<VariantCategory>;

#[derive(Deserialize, Serialize, Clone)]
pub struct VariantCategory {
    pub category: String,
    pub variants: Vec<Variant>
}

#[derive(Deserialize, Serialize, Clone)]
pub struct VariantInformation {
    pub name: String,
    pub stock: StockList,
    pub images: Vec<Url>,
    pub retail_price: f32,
    pub marginal_price: f32,
    pub id: String,
    pub loyalty_discount: DiscountValue,
    /// The group codes for all sub-variants; i.e. is White, Short Sleeve and Small.
    pub variant_code: VariantIdTag,
    pub order_history: HistoryList,
    pub stock_information: StockInformation,
    pub barcode: String
}

impl Display for VariantInformation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "\t{} ({:?}) ${}[R-:-M]${}", 
            self.name, self.variant_code, self.retail_price, self.marginal_price 
        )
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Promotion {
    pub id: Id,
    pub name: String,
    pub buy: PromotionBuy,
    pub get: PromotionGet,
    pub valid_till: DateTime<Utc>,
    pub timestamp: DateTime<Utc>
}

#[derive(Deserialize, Serialize, Clone)]
pub struct PromotionInput {
    name: String,
    buy: PromotionBuy,
    get: PromotionGet,
    valid_till: DateTime<Utc>,
    timestamp: DateTime<Utc>
}

#[derive(Deserialize, Serialize, Clone)]
pub enum PromotionBuy {
    // This(quantity), Specific((id, quantity)), Any(quantity)
    Specific((String, f32)), Any(f32), Category((String, f32))
}

#[derive(Deserialize, Serialize, Clone)]
pub enum PromotionGet {
    /// `SoloThis(discount)` <br />
    /// *Represents the individual product.* <br /> <br />
    /// Is used in cases where the product is the recipient of the promotion in inclusive quantity, i.e. 50% off t-shirts (applies to self)
    SoloThis(DiscountValue),
    /// `This((quantity, discount))` <br />
    /// *Represents the continual product.* <br /> <br />
    /// Applies when the following product is the recipient of the promotion, i.e. Buy 1 get 1 half price (product receives 50% discount, but is not directly matching the GET criterion (quantity >= 2...))
    This((f32, DiscountValue)),
    /// `Specific((sku, (quantity, discount)))` <br />
    /// *Represents a specific product* <br /> <br />
    ///  Is used to reference a specific product by its SKU, i.e. Buy any 1 product, get a lib balm $5 off.
    Specific((String, (f32, DiscountValue))),
    /// `Any((quantity, discount))` <br />
    /// *Represents all products* <br /> <br />
    /// A general match-any clause to refer to any product, i.e. Buy 1 get any other product $5 off.
    Any((f32, DiscountValue)),
    /// `Category(category, (quantity, discount))` <br />
    /// *Represents all products within a category* <br /> <br />
    /// Matches any product within a category, the category is referenced in a products `TagList`. I.e. Buy any 1 product, get any t-shirt 20% off. 
    Category((String, (f32, DiscountValue)))
}

impl Promotion {
    pub async fn insert(prm: PromotionInput, db: &DbConn) -> Result<InsertResult<promotion::ActiveModel>, DbErr> {
        let id = Uuid::new_v4().to_string();

        let insert_crud = promotion::ActiveModel {
            id: Set(id.to_string()),
            name: Set(prm.name.to_string()),
            buy: Set(json!(prm.buy)),
            get: Set(json!(prm.get)),
            valid_till: Set(prm.valid_till.naive_utc()),
            timestamp: Set(prm.timestamp.naive_utc()),
        };

        match Promotions::insert(insert_crud).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err)
        }
    }

    pub async fn fetch_by_id(id: &str, db: &DbConn) -> Result<Promotion, DbErr> {
        let pdt = Promotions::find_by_id(id.to_string()).one(db).await?;
        let p = pdt.unwrap();

        Ok(Promotion { 
            id: p.id, 
            name: p.name, 
            buy: serde_json::from_value::<PromotionBuy>(p.buy).unwrap(), 
            get: serde_json::from_value::<PromotionGet>(p.get).unwrap(), 
            valid_till: DateTime::from_utc(p.valid_till, Utc), 
            timestamp: DateTime::from_utc(p.timestamp, Utc) 
        })
    }

    pub async fn fetch_by_query(query: &str, db: &DbConn) -> Result<Vec<Promotion>, DbErr> {
        let res = Promotions::find()
            // Is the bought product
            .having(promotion::Column::Buy.contains(query))
            // Is the promoted product
            .having(promotion::Column::Get.contains(query))
            // Meets the Any criterion
            .having(promotion::Column::Buy.contains("Any"))
            // Meets the Any criterion
            .having(promotion::Column::Get.contains("Any"))
            .all(db).await?;

        let mapped = res.iter().map(|p| {
            Promotion { 
                id: p.id.clone(), 
                name: p.name.clone(), 
                buy: serde_json::from_value::<PromotionBuy>(p.buy.clone()).unwrap(), 
                get: serde_json::from_value::<PromotionGet>(p.get.clone()).unwrap(), 
                valid_till: DateTime::from_utc(p.valid_till, Utc), 
                timestamp: DateTime::from_utc(p.timestamp, Utc) 
            }
        }).collect();

        Ok(mapped)
    }

    pub async fn update(prm: PromotionInput, id: &str, db: &DbConn) -> Result<Promotion, DbErr> {
        promotion::ActiveModel {
            id: Set(id.to_string()),
            name: Set(prm.name.to_string()),
            buy: Set(json!(prm.buy)),
            get: Set(json!(prm.get)),
            valid_till: Set(prm.valid_till.naive_utc()),
            timestamp: Set(prm.timestamp.naive_utc()),
        }.update(db).await?;

        Self::fetch_by_id(id, db).await
    }

    pub async fn fetch_all(db: &DbConn) -> Result<Vec<Promotion>, DbErr> {
        let stores = Promotions::find().all(db).await?;

        let mapped = stores.iter().map(|e| 
            Promotion { 
                id: e.id.clone(), 
                name: e.name.clone(),
                buy: serde_json::from_value::<PromotionBuy>(e.buy.clone()).unwrap(), 
                get: serde_json::from_value::<PromotionGet>(e.get.clone()).unwrap(), 
                timestamp: DateTime::from_utc(e.timestamp, Utc),
                valid_till: DateTime::from_utc(e.valid_till, Utc),
            }
        ).collect();
        
        Ok(mapped)
    }

    pub async fn insert_many(stores: Vec<PromotionInput>, db: &DbConn) -> Result<InsertResult<promotion::ActiveModel>, DbErr> {
        let entities = stores.into_iter().map(|prm| {
            let id = Uuid::new_v4().to_string();

            promotion::ActiveModel {
                id: Set(id.to_string()),
                name: Set(prm.name.to_string()),
                buy: Set(json!(prm.buy)),
                get: Set(json!(prm.get)),
                valid_till: Set(prm.valid_till.naive_utc()),
                timestamp: Set(prm.timestamp.naive_utc()),
            }
        });

        match Promotions::insert_many(entities).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err)
        }
    }

    pub async fn generate(db: &DbConn) -> Result<Vec<Promotion>, DbErr> {
        let promotions = example_promotions();

        match Promotion::insert_many(promotions, db).await {
            Ok(_) => {
                match Promotion::fetch_all(db).await {
                    Ok(res) => {
                        Ok(res)
                    },  
                    Err(e) => Err(e)
                }
            },
            Err(e) => Err(e)
        }
    }
}

/// Represents all sub-variant types; i.e. All 'White' variants, whether small, long-sleeve, ... it represents the sub-group of all which are 'White'.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Variant {
    pub name: String,
    pub images: Vec<Url>,
    pub marginal_price: f32,
    pub variant_code: String,
    pub order_history: HistoryList,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct StockInformation {
    pub stock_group: String,
    pub sales_group: String,
    pub value_stream: String,

    pub brand: String,
    pub unit: String,

    /// Non-required field which outlines the tax code of the product if necessary.
    pub tax_code: String,

    /// The variant's weight in kilograms.
    pub weight: String,

    /// The volume of the product in meters cubed, kept specific to each variant.
    pub volume: String,

    /// A quantity considered to be the *maximum*. If the quantity dips below such value, it is suggested a restock should take place.
    pub max_volume: String,

    /// If the product's supply cannot be fulfilled at the current time, due to a lack of availability. 
    /// 
    /// By setting `back_order` to `true`, it allows for the purchase of the product on the promise it will be delivered to the customer or collected from the store at a later date. **This must be made clear and known to the customer.**
    pub back_order: bool,
    /// A product which is no longer source-able. Once the product's inventory is consumed it is indicated to never be replenished.
    pub discontinued: bool,
    /// A `non_diminishing` product is often a service rather than a product, i.e. freight. It is **not removed** from inventory upon consumption, rather attached.
    pub non_diminishing: bool,
    /// A non-shippable good is one which cannot be dispatched between stores or sent to a customers home, this might be a fragile product, service, oversized good or edge case.
    pub shippable: bool
}

impl Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "\t{} ({}) ${}", 
            self.name, self.variant_code, self.marginal_price 
        )
    }
}

fn example_promotions() -> Vec<PromotionInput> {
    vec![
        PromotionInput { 
            name: format!("Buy 1 Get 1 10% off"), 
            buy: PromotionBuy::Any(1.0), 
            get: PromotionGet::Any((1.0, DiscountValue::Percentage(10))), 
            valid_till: Utc::now().checked_add_days(Days::new(7)).unwrap(), 
            timestamp: Utc::now()
        },
        PromotionInput { 
            name: format!("50% off T-shirts"), 
            buy: PromotionBuy::Category(("Tee".into(), 1.0)), 
            get: PromotionGet::SoloThis(DiscountValue::Percentage(50)), 
            valid_till: Utc::now().checked_add_days(Days::new(7)).unwrap(), 
            timestamp: Utc::now()
        },
        PromotionInput { 
            name: format!("Buy a Kayak, get a Life Jacket 50% off"), 
            buy: PromotionBuy::Specific(("654321".into(), 1.0)), 
            get: PromotionGet::Specific(("162534".into(), (1.0, DiscountValue::Percentage(50)))), 
            valid_till: Utc::now().checked_add_days(Days::new(7)).unwrap(), 
            timestamp: Utc::now()
        }
    ]
}