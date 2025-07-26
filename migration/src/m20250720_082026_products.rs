use sea_orm_migration::{prelude::*, schema::*};
use crate::m20250723_074116_users::Users;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .create_table(
                Table::create()
                    .table(Products::Table)
                    .if_not_exists()
                    .col(pk_uuid(Products::Id).uuid())
                    .col(string(Products::Sku))
                    .col(string(Products::Name))
                    .col(uuid(Products::UserId))
                    .col(string_uniq(Products::Slug))
                    .col(text(Products::Description))
                    .col(big_unsigned(Products::Price))
                    .col(big_unsigned(Products::Quantity))
                    .col(timestamp_with_time_zone(Products::CreatedAt)
                        .default(Expr::current_timestamp()
                        ))
                    .col(timestamp_with_time_zone_null(Products::UpdatedAt))
                    .col(timestamp_with_time_zone_null(Products::PublishedAt))
                    .to_owned(),
            )
            .await?;

        manager.create_foreign_key(
            ForeignKey::create()
                .from_tbl(Products::Table)
                .from_col(Products::UserId)
                .to_tbl(Users::Table)
                .to_col(Users::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned()
        )
            .await?;

        manager.create_index(
            Index::create()
                .if_not_exists()
                .table(Products::Table)
                .col(Products::Name).name("product-name")
                .col(Products::Slug).name("product-slug")
                .col(Products::CreatedAt).name("product-createdAt")
                .to_owned(),
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .drop_table(Table::drop().table(Products::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Products {
    Table,
    Id,
    Sku,
    UserId,
    Name,
    Slug,
    Description,
    Price,
    Quantity,
    CreatedAt,
    UpdatedAt,
    PublishedAt
}
