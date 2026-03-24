pub use sea_orm_migration::prelude::*;

pub struct Migrator;

pub mod m20250723_074116_users;
mod m20250725_111229_roles;
mod m20250725_112514_user_roles;
mod m20260213_114308_books;
mod m20260213_120143_book_files;
mod m20260227_112928_book_loans;




#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250723_074116_users::Migration),
            Box::new(m20250725_111229_roles::Migration),
            Box::new(m20250725_112514_user_roles::Migration),
            Box::new(m20260213_114308_books::Migration),
            Box::new(m20260213_120143_book_files::Migration),
            Box::new(m20260227_112928_book_loans::Migration),
        ]
    }
}

