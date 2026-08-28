use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("database error")]
    Sqlite(#[from] rusqlite::Error),
    #[error("storage unavailable")]
    Unavailable,
    #[error("database lock poisoned")]
    Poisoned,
    #[error("item not found")]
    NotFound,
}

impl DbError {
    pub fn to_user_message(&self) -> String {
        match self {
            Self::NotFound => "مورد پیدا نشد".to_string(),
            Self::Unavailable => "ذخیره‌سازی در دسترس نیست".to_string(),
            Self::Sqlite(_) | Self::Poisoned => "خطا در پایگاه داده".to_string(),
        }
    }
}

pub type DbResult<T> = Result<T, DbError>;
