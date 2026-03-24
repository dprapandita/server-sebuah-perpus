use crate::app::models::book_loans::{
    BookLoan, BookLoanAggregate, BookLoanPayloadRequest, BookLoanUpdatePayload,
};
use crate::core::error::AppError;
use crate::utils::capitalize_first;
use async_trait::async_trait;
use entity::generated::sea_orm_active_enums::BookStatusLoanType;
use sea_orm::{DatabaseConnection, Set};
use std::sync::Arc;
use uuid::Uuid;

pub fn parse_loan_type(v: &str) -> Result<BookStatusLoanType, AppError> {
    // Membersihkan spasi dan menyamakan huruf kapital di awal
    match capitalize_first(v.trim()).as_str() {
        // Bisa menerima "Dipinjamkan" atau "Aktif"
        "Dipinjamkan" | "Aktif" | "Active" => Ok(BookStatusLoanType::Active),

        // Bisa menerima "Terlambat" atau "Overdue"
        "Terlambat" | "Overdue" => Ok(BookStatusLoanType::Overdue),

        // Bisa menerima "Dikembalikan" atau "Selesai"
        "Dikembalikan" | "Selesai" | "Returned" => Ok(BookStatusLoanType::Returned),

        // Bisa menerima "Hilang"
        "Hilang" | "Lost" => Ok(BookStatusLoanType::Lost),

        // Jika input tidak cocok dengan daftar di atas, kembalikan Error
        _ => Err(AppError::BadRequest(format!(
            "Status peminjaman '{}' tidak valid. Gunakan: Dipinjamkan, Terlambat, Dikembalikan, atau Hilang.", v
        ))),
    }
}

#[async_trait]
pub trait BookLoanRepository: Send + Sync {
    async fn create_loan(
        &self,
        trx: &sea_orm::DatabaseTransaction,
        book_loan_payload: BookLoanPayloadRequest,
    ) -> Result<BookLoan, AppError>;
    async fn update_loan(
        &self,
        trx: &sea_orm::DatabaseTransaction,
        loan_id: Uuid,
        new_loan: BookLoanUpdatePayload,
    ) -> Result<BookLoan, AppError>;
    async fn find_loan_by_id(&self, loan_id: Uuid) -> Result<Option<BookLoan>, AppError>;
    async fn find_loan_history(&self, user_id: Uuid) -> Result<Vec<BookLoanAggregate>, AppError>;
}

pub struct SeaORMBookLoanRepository {
    pub db: Arc<DatabaseConnection>,
}

impl SeaORMBookLoanRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl BookLoanRepository for SeaORMBookLoanRepository {
    async fn create_loan(
        &self,
        trx: &sea_orm::DatabaseTransaction,
        book_loan_payload: BookLoanPayloadRequest,
    ) -> Result<BookLoan, AppError> {
        use sea_orm::ActiveModelTrait;

        let borrow_dt = book_loan_payload
            .borrow_date
            .unwrap_or_else(|| chrono::Utc::now().naive_utc().date())
            .and_hms_opt(0, 0, 0)
            .unwrap();

        let due_dt = book_loan_payload.due_date.and_hms_opt(23, 59, 59).unwrap();

        let new_load = entity::book_loans::ActiveModel {
            id: Set(Uuid::new_v4()),
            book_id: Set(book_loan_payload.book_id),
            user_id: Set(book_loan_payload.user_id),
            due_date: Set(due_dt),
            borrow_date: Set(borrow_dt),
            ..Default::default()
        };

        let result = new_load.insert(trx).await?;

        // Convert to BookLoan. We can't use From<BookLoanAggregate> here,
        // so we map manually.
        Ok(BookLoan {
            book_loan_id: result.id,
            book_id: result.book_id,
            user_id: result.user_id,
            borrow_date: result.borrow_date,
            due_date: result.due_date,
            return_date: result.returned_at,
            status: result.status,
        })
    }

    async fn update_loan(
        &self,
        trx: &sea_orm::DatabaseTransaction,
        loan_id: Uuid,
        payload: BookLoanUpdatePayload,
    ) -> Result<BookLoan, AppError> {
        use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};
        let loan = entity::book_loans::Entity::find_by_id(loan_id)
            .one(trx)
            .await?
            .ok_or(AppError::NotFound)?;

        let mut active_model = loan.into_active_model();
        if let Some(returned_date) = payload.returned_date {
            active_model.returned_at = Set(Some(returned_date));
        }
        if let Some(due_date) = payload.due_date {
            active_model.due_date = Set(due_date);
        }
        if let Some(status) = payload.status {
            active_model.status = Set(Some(status));
        }

        let result = active_model.update(trx).await?;
        Ok(BookLoan {
            book_loan_id: result.id,
            book_id: result.book_id,
            user_id: result.user_id,
            borrow_date: result.borrow_date,
            due_date: result.due_date,
            return_date: result.returned_at,
            status: result.status,
        })
    }

    async fn find_loan_by_id(&self, loan_id: Uuid) -> Result<Option<BookLoan>, AppError> {
        use sea_orm::EntityTrait;
        let loan = entity::book_loans::Entity::find_by_id(loan_id)
            .one(self.db.as_ref())
            .await?;

        Ok(loan.map(|result| BookLoan {
            book_loan_id: result.id,
            book_id: result.book_id,
            user_id: result.user_id,
            borrow_date: result.borrow_date,
            due_date: result.due_date,
            return_date: result.returned_at,
            status: result.status,
        }))
    }

    async fn find_loan_history(&self, user_id: Uuid) -> Result<Vec<BookLoanAggregate>, AppError> {
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
        let loans = entity::book_loans::Entity::find()
            .filter(entity::book_loans::Column::UserId.eq(user_id))
            .find_also_related(entity::books::Entity)
            .all(self.db.as_ref())
            .await?;

        // We also need to get the user data for BookLoanAggregate, but to save query complexity
        // let's fetch the user separately or change BookLoanAggregate

        let mut aggregates = Vec::new();
        for (loan, book_opt) in loans {
            if let Some(book) = book_opt {
                // Fetch user
                let user = entity::users::Entity::find_by_id(loan.user_id)
                    .one(self.db.as_ref())
                    .await?
                    .ok_or(AppError::NotFound)?;

                aggregates.push(BookLoanAggregate {
                    book_loan: loan,
                    book,
                    user,
                });
            }
        }
        Ok(aggregates)
    }
}
