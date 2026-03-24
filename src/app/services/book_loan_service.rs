use chrono::Utc;
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::sync::Arc;
use uuid::Uuid;

use crate::app::models::book_loans::{
    BookLoan, BookLoanAggregate, BookLoanPayloadRequest, BookLoanUpdatePayload,
};
use crate::app::repositories::book_loan_repository::BookLoanRepository;
use crate::app::repositories::book_repository::BookRepository;
use crate::core::error::AppError;
use entity::generated::sea_orm_active_enums::{BookStatusLoanType, BookStatusType};

pub struct BookLoanService {
    db: Arc<DatabaseConnection>,
    loan_repo: Arc<dyn BookLoanRepository>,
    book_repo: Arc<dyn BookRepository>,
}

impl BookLoanService {
    pub fn new(
        db: Arc<DatabaseConnection>,
        loan_repo: Arc<dyn BookLoanRepository>,
        book_repo: Arc<dyn BookRepository>,
    ) -> Self {
        Self {
            db,
            loan_repo,
            book_repo,
        }
    }

    pub async fn borrow_book(&self, payload: BookLoanPayloadRequest) -> Result<BookLoan, AppError> {
        let trx = self.db.begin().await?;

        // 1. Check if book exists and is available for loan
        let book = self
            .book_repo
            .find_by_id(payload.book_id)
            .await?
            .ok_or(AppError::NotFound)?;

        // Assuming a book is only available if it's 'Published' and its loan status is NOT 'Active'
        if book.status != Some(BookStatusType::Published) {
            return Err(AppError::BadRequest(
                "Buku tidak tersedia untuk dipinjam.".to_string(),
            ));
        }

        if book.status_loan != Some(BookStatusLoanType::Active) {
            return Err(AppError::BadRequest(
                "Buku sedang dipinjam oleh orang lain.".to_string(),
            ));
        }

        // 2. Create the loan record
        let loan = self.loan_repo.create_loan(&trx, payload).await?;

        // 3. Update the book's loan status to 'Active'
        self.book_repo
            .update_status_loan(&trx, book.id, Some(BookStatusLoanType::Active))
            .await?;

        trx.commit().await?;

        Ok(loan)
    }

    pub async fn return_book(&self, loan_id: Uuid, user_id: Uuid) -> Result<BookLoan, AppError> {
        let trx = self.db.begin().await?;

        // 1. Fetch loan
        let loan = self
            .loan_repo
            .find_loan_by_id(loan_id)
            .await?
            .ok_or(AppError::NotFound)?;

        if loan.user_id != user_id {
            return Err(AppError::Forbidden);
        }

        if loan.status == Some(BookStatusLoanType::Returned) {
            return Err(AppError::BadRequest(
                "Buku ini sudah dikembalikan.".to_string(),
            ));
        }

        // 2. Update loan to 'Returned', set returned_at date
        let returned_date = Utc::now().naive_utc();

        let update_payload = BookLoanUpdatePayload {
            book_id: loan.book_id,
            user_id: loan.user_id,
            returned_date: Some(returned_date),
            due_date: None,
            status: Some(BookStatusLoanType::Returned),
        };

        let updated_loan = self
            .loan_repo
            .update_loan(&trx, loan_id, update_payload)
            .await?;

        // 3. Update book's loan status to Returned/None indicating availability
        self.book_repo
            .update_status_loan(&trx, loan.book_id, Some(BookStatusLoanType::Returned))
            .await?;

        trx.commit().await?;

        Ok(updated_loan)
    }

    pub async fn get_loan_history(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<BookLoanAggregate>, AppError> {
        self.loan_repo.find_loan_history(user_id).await
    }
}
