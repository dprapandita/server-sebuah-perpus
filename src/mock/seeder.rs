use crate::app::hashing::hash;
use crate::core::error::AppError;
use crate::mock::factory::{RoleFactory, UserFactory};
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait, QueryFilter, Set, TransactionTrait};
use uuid::Uuid;

pub async fn seed_all(db: &DatabaseConnection) -> Result<(), AppError> {
    tracing::info!("Memulai proses seeding database...");
    let txn = db.begin().await?;

    tracing::info!("Seed roles");
    seed_roles(&txn).await?;

    tracing::info!("Seed admin user");
    seed_admin_user(&txn).await?;

    txn.commit().await?;
    tracing::info!("Proses seeding selesai.");
    Ok(())
}

async fn seed_admin_user(db: &DatabaseTransaction) -> Result<Uuid, sea_orm::DbErr> {
    // cek apakah sudah ada admin user
    if let Some(model) = entity::users::Entity::find()
        .filter(entity::users::Column::Username.eq("superadmin"))
        .one(db)
        .await?
    {
        return Ok(model.id);
    }

    // Hash password using bcrypt to avoid stack overflow with Argon2
    let password_hash = hash::hash_password(&std::env::var("ADMIN_PASSWORD").unwrap_or_default())
        .expect("Failed to hash password");

    let admin_id = Uuid::new_v4();

    let admin = UserFactory::new()
        .id(admin_id)
        .username("superadmin".to_string())
        .name("Super Admin".to_string())
        .email("superadmin@example.com".to_string())
        .password(password_hash)
        .build()
        .insert(db)
        .await?;

    // Attach admin role to admin user using user_roles pivot table
    let admin_role = entity::roles::Entity::find()
        .filter(entity::roles::Column::Name.eq("admin"))
        .one(db)
        .await?
        .expect("admin role must exist");

    use entity::user_roles;
    let user_role_model = user_roles::ActiveModel {
        user_id: Set(admin.id),
        role_id: Set(admin_role.id),
        ..Default::default()
    };

    user_roles::Entity::insert(user_role_model)
        .on_conflict(
            OnConflict::columns([user_roles::Column::UserId, user_roles::Column::RoleId])
                .do_nothing()
                .to_owned(),
        )
        .exec(db)
        .await?;

    tracing::info!("Superadmin berhasil di-seed dengan ID: {}", admin.id);

    Ok(admin.id)
}

pub async fn seed_roles(db: &DatabaseTransaction) -> Result<(), AppError> {
    let roles = vec!["admin", "manager", "staff", "sales", "guest"];

    let mut role_models = Vec::new();

    for name in roles {
        // Buat modelnya tanpa perlu mengecek ke database
        role_models.push(RoleFactory::new().name(name.to_string()).build());
    }

    // Insert semua sekaligus.
    // Jika nama role sudah ada di database, abaikan saja (do_nothing).
    let result_seeding = entity::roles::Entity::insert_many(role_models)
        .on_conflict(
            OnConflict::column(entity::roles::Column::Name)
                .do_nothing()
                .to_owned(),
        )
        .exec(db)
        .await;

    match result_seeding {
        Ok(_) => {
            println!("✅ Seeding berhasil: Role baru ditambahkan ke database.");
        }
        Err(DbErr::RecordNotInserted) => {
            println!("✅ Seeding dilewati: Semua role sudah tersedia di database.");
        }
        Err(e) => {
            panic!("❌ Gagal menjalankan seeding database: {}", e);
        }
    }

    Ok(())
}

async fn ensure_role(
    db: &DatabaseTransaction,
    name: &str,
) -> Result<entity::roles::Model, AppError> {
    if let Some(r) = entity::roles::Entity::find()
        .filter(entity::roles::Column::Name.eq(name))
        .one(db)
        .await?
    {
        return Ok(r);
    }
    let role = entity::roles::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(name.to_string()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(role)
}

async fn attach_roles(
    db: &DatabaseTransaction,
    user_id: Uuid,
    role_names: &[&str],
) -> Result<(), sea_orm::DbErr> {
    for role_name in role_names {
        let role = ensure_role(db, role_name)
            .await
            .map_err(|e| sea_orm::DbErr::Custom("Error ensure role".to_string()))?;

        let exists = entity::user_roles::Entity::find()
            .filter(entity::user_roles::Column::UserId.eq(user_id))
            .filter(entity::user_roles::Column::RoleId.eq(role.id))
            .one(db)
            .await?
            .is_some();

        if !exists {
            let role_model = entity::user_roles::ActiveModel {
                user_id: Set(user_id),
                role_id: Set(role.id),
                ..Default::default()
            };

            entity::user_roles::Entity::insert(role_model)
                .on_conflict(
                    OnConflict::columns([
                        entity::user_roles::Column::UserId,
                        entity::user_roles::Column::RoleId,
                    ])
                    .do_nothing()
                    .to_owned(),
                )
                .exec(db)
                .await?;
        }
    }
    Ok(())
}
