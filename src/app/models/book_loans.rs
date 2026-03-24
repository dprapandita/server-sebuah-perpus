use chrono::{NaiveDate, NaiveDateTime};
use entity::generated::sea_orm_active_enums::{BookStatusLoanType, BookStatusType};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct BookLoan {
    pub book_loan_id: Uuid,
    pub book_id: Uuid,
    pub user_id: Uuid,
    pub borrow_date: NaiveDateTime,
    pub due_date: NaiveDateTime,
    pub return_date: Option<NaiveDateTime>,
    pub status: Option<BookStatusLoanType>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BookLoanAggregate {
    pub book: entity::books::Model,
    pub book_loan: entity::book_loans::Model,
    pub user: entity::users::Model,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BookLoanPayloadRequest {
    pub book_id: Uuid,
    pub user_id: Uuid,
    #[serde(default)]
    pub borrow_date: Option<NaiveDate>,
    pub due_date: NaiveDate,
    #[serde(default)]
    pub status: Option<BookStatusType>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BookLoanUpdatePayload {
    pub book_id: Uuid,
    pub user_id: Uuid,
    pub returned_date: Option<NaiveDateTime>,
    pub due_date: Option<NaiveDateTime>,
    pub status: Option<BookStatusLoanType>,
}

impl From<BookLoanAggregate> for BookLoan {
    fn from(aggregate: BookLoanAggregate) -> Self {
        Self {
            book_loan_id: aggregate.book_loan.id,
            book_id: aggregate.book.id,
            user_id: aggregate.user.id,
            borrow_date: aggregate.book_loan.borrow_date,
            due_date: aggregate.book_loan.due_date,
            return_date: aggregate.book_loan.returned_at,
            status: aggregate.book_loan.status,
        }
    }
}
