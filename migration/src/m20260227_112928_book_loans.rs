use sea_orm_migration::{prelude::*, schema::*};
use crate::m20250723_074116_users::Users;
use crate::m20260213_114308_books::{BookStatusLoan, Books};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .create_table(
                Table::create()
                    .table(BookLoans::Table)
                    .if_not_exists()
                    .col(pk_uuid(BookLoans::Id))
                    .col(uuid(BookLoans::BookId))
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("book-loans")
                            .from(BookLoans::Table, BookLoans::BookId)
                            .to(Books::Table, Books::Id)
                    )
                    .col(uuid(BookLoans::UserId))
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("user-loans")
                            .from(BookLoans::Table, BookLoans::UserId)
                            .to(Users::Table, Users::Id)
                    )
                    .col(timestamp(BookLoans::BorrowDate))
                    .col(timestamp(BookLoans::DueDate))
                    .col(timestamp_null(BookLoans::ReturnedAt))
                    .col(ColumnDef::new(BookLoans::Status)
                        .custom(BookStatusLoan::BookStatusLoanType))
                    .col(timestamp(BookLoans::CreatedAt)
                        .default(Expr::current_timestamp()))
                    .col(timestamp_null(BookLoans::UpdatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .drop_table(Table::drop().table(BookLoans::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum BookLoans {
    Table,
    Id,
    BookId,
    UserId,
    BorrowDate,
    DueDate,
    ReturnedAt,
    Status,
    CreatedAt,
    UpdatedAt,
}
