use sea_orm_migration::{prelude::*, schema::*};
use crate::m20250723_074116_users::Users;
use crate::m20250725_111229_roles::Role;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .create_table(
                Table::create()
                    .table(UserRoles::Table)
                    .if_not_exists()
                    .primary_key(
                        Index::create()
                            .col(UserRoles::UserId).name("user_roles-user_id")
                            .col(UserRoles::RoleId).name("user_roles-role_id")
                    )
                    .col(uuid(UserRoles::UserId))
                    .col(uuid(UserRoles::RoleId))
                    .col(timestamp_with_time_zone_null(UserRoles::CreatedAt))
                    .col(timestamp_with_time_zone_null(UserRoles::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserRoles::Table, UserRoles::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserRoles::Table, UserRoles::RoleId)
                            .to(Role::Table, Role::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .drop_table(Table::drop().table(UserRoles::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UserRoles {
    Table,
    UserId,
    RoleId,
    CreatedAt,
    UpdatedAt
}
