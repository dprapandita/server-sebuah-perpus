use sea_orm_migration::{prelude::*, schema::*};
use crate::m20260213_114308_books::Books;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .create_table(
                Table::create()
                    .table(BookFiles::Table)
                    .if_not_exists()
                    .col(pk_uuid(BookFiles::Id))
                    .col(uuid(BookFiles::BookId))
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .from(BookFiles::Table, BookFiles::BookId)
                            .to(Books::Table, Books::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade)
                    )
                    .col(string(BookFiles::FilePath))
                    .col(string(BookFiles::FileFormat))
                    .col(big_integer(BookFiles::FileSize))
                    .col(string(BookFiles::Checksum))
                    .col(timestamp(BookFiles::CreatedAt).default(
                        Expr::current_timestamp()
                    ))
                    .to_owned(),
            )
            .await?;
        manager.create_index(
            Index::create()
                .name("idx-book-id")
                .table(BookFiles::Table)
                .col(BookFiles::BookId)
                .to_owned()
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .drop_table(Table::drop().table(BookFiles::Table).to_owned())
            .await?;
        manager.drop_index(
            Index::drop()
                .if_exists()
                .name("idx-book-id")
                .table(BookFiles::Table)
                .to_owned()
        ).await
    }
}

#[derive(DeriveIden)]
enum BookFiles {
    Table,
    Id,
    BookId,
    FilePath,
    FileFormat,
    FileSize,
    Checksum,
    CreatedAt
}
