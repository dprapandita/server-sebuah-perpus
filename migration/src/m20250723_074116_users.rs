use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(pk_uuid(Users::Id).uuid())
                    .col(string(Users::Name))
                    .col(string_uniq(Users::Username))
                    .col(string_uniq(Users::Email))
                    .col(string(Users::Password))
                    .col(timestamp_with_time_zone_null(Users::CreatedAt))
                    .col(timestamp_with_time_zone_null(Users::UpdatedAt))
                    .to_owned(),
            )
            .await?;
        manager.create_index(
            Index::create()
                .name("idx-username")
                .table(Users::Table)
                .col(Users::Username)
                .to_owned(),
        ).await?;
        manager.create_index(
            Index::create()
                .name("idx-email")
                .table(Users::Table)
                .col(Users::Email)
                .to_owned(),
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;
        manager.drop_index(
            Index::drop()
                .if_exists()
                .name("idx-username")
                .table(Users::Table)
                .to_owned()
        ).await?;
        manager.drop_index(
            Index::drop()
                .if_exists()
                .name("idx-email")
                .table(Users::Table)
                .to_owned()
        ).await
    }
}

#[derive(DeriveIden)]
pub enum Users {
    Table,
    Id,
    Name,
    Username,
    Email,
    Password,
    CreatedAt,
    UpdatedAt,
}
