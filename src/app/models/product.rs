use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub price: i64,
    pub quantity: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProductPayload {
    pub sku: String,
    pub name: String,
    pub description: String,
    pub price: i64,
    pub quantity: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProductPayload {
    pub sku: String,
    pub name: String,
    pub description: String,
    pub price: i64,
    pub quantity: i64,
}