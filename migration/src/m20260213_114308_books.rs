use sea_orm_migration::{prelude::*, schema::*};
use crate::extension::postgres::{Type, TypeCreateStatement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_type(
            TypeCreateStatement::new()
                .values([BookStatus::Archived, BookStatus::Previewing, BookStatus::Published, BookStatus::Draft, BookStatus::Takedown])
                .as_enum(BookStatus::BookStatusType)
                .to_owned()
        ).await?;
        manager.create_type(
            TypeCreateStatement::new()
                .values([BookStatusLoan::Active, BookStatusLoan::Overdue, BookStatusLoan::Returned, BookStatusLoan::Lost])
                .as_enum(BookStatusLoan::BookStatusLoanType)
                .to_owned()
        ).await?;
        manager
            .create_table(
                Table::create()
                    .table(Books::Table)
                    .if_not_exists()
                    .col(pk_uuid(Books::Id))
                    .col(string_len_null(Books::Isbn, 20))
                    .col(string(Books::Title))
                    .col(string_uniq(Books::Slug))
                    .col(text(Books::Description))
                    .col(string(Books::Author))
                    .col(string(Books::Publisher))
                    .col(string_null(Books::CoverImage))
                    .col(ColumnDef::new(Books::Status)
                        .custom(BookStatus::BookStatusType))
                    .col(ColumnDef::new(Books::StatusLoan)
                        .custom(BookStatusLoan::BookStatusLoanType))
                    .col(timestamp(Books::CreatedAt).default(
                        Expr::current_timestamp()
                    ))
                    .col(timestamp_null(Books::UpdatedAt))
                    .to_owned(),
            )
            .await?;
        manager.create_index(
            Index::create()
                .name("idx-slug-book")
                .table(Books::Table)
                .col(Books::Slug)
                .to_owned()
        ).await?;
        manager.create_index(
            Index::create()
                .name("idx-isbn-book")
                .table(Books::Table)
                .col(Books::Isbn)
                .to_owned()
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .drop_table(Table::drop().table(Books::Table).to_owned())
            .await?;
        manager.drop_index(
            Index::drop()
                .if_exists()
                .name("idx-isbn-book")
                .table(Books::Table)
                .to_owned()
        ).await?;
        manager.drop_index(
            Index::drop()
                .if_exists()
                .name("idx-slug-book")
                .table(Books::Table)
                .to_owned()
        ).await?;
        manager.drop_type(
            Type::drop()
                .if_exists()
                .name("book_status_type")
                .to_owned()
        ).await?;
        manager.drop_type(
            Type::drop()
                .if_exists()
                .name("book_status_loan_type")
                .to_owned()
        ).await
    }
}

#[derive(DeriveIden)]
pub enum Books {
    Table,
    Id,
    Isbn,
    Title,
    Slug,
    Description,
    Publisher,
    Author,
    CoverImage,
    Status,
    StatusLoan,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum BookStatus {
    BookStatusType,
    Draft,
    Published,
    Archived,
    Takedown,
    Previewing
}

#[derive(DeriveIden)]
pub enum BookStatusLoan {
    BookStatusLoanType,
    Active,
    Returned,
    Overdue,
    Lost
}