use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .create_table(
                Table::create()
                    .table(Roles::Table)
                    .if_not_exists()
                    .col(pk_uuid(Roles::Id).uuid())
                    .col(string(Roles::Name))
                    .col(timestamp_with_time_zone_null(Roles::CreatedAt))
                    .col(timestamp_with_time_zone_null(Roles::UpdatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .drop_table(Table::drop().table(Roles::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Roles {
    Table,
    Id,
    Name,
    CreatedAt,
    UpdatedAt,
}
