use std::sync::Arc;
use crate::app::models::product::{CreateProductPayload, Product, UpdateProductPayload};
use crate::app::repositories::product_repository::ProductRepository;
use crate::core::error::AppError;

pub struct ProductService {
    repo: Arc<dyn ProductRepository>,
}

impl ProductService {
    pub fn new(repo: Arc<dyn ProductRepository>) -> ProductService {
        Self { repo }
    }

    pub async fn create_product(&self, payload: CreateProductPayload ) -> Result<Product, AppError>
    {
        self.repo.create(payload).await
    }

    pub async fn get_all_products(&self) -> Result<Vec<Product>, AppError>
    {
        self.repo.find_all().await
    }

    pub async fn get_product_by_slug(&self, slug: &str) -> Result<Product, AppError>
    {
        self.repo.find_by_slug(slug).await?.ok_or(AppError::NotFound)
    }

    pub async fn update_product(&self, slug: &str, payload: UpdateProductPayload)
        -> Result<Product, AppError>
    {
        self.repo.update(slug, payload).await
    }

    pub async fn delete_product(&self, slug: &str) -> Result<(), AppError>
    {
        self.repo.delete(slug).await
    }
}