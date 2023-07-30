//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.3

use super::sea_orm_active_enums::TransactionType;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "Transactions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub customer: Json,
    pub transaction_type: TransactionType,
    pub products: Json,
    pub order_total: f32,
    pub payment: Json,
    pub order_date: DateTime,
    pub order_notes: Json,
    #[sea_orm(column_type = "Text")]
    pub salesperson: String,
    #[sea_orm(column_type = "Text")]
    pub kiosk: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
