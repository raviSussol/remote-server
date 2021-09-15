use crate::database::{
    repository::{
        macros::{execute, first, load},
        RepositoryError,
    },
    schema::{diesel_schema::user_account::dsl::*, UserAccountRow},
};
use diesel::prelude::*;

use super::DbConnectionPool;

pub struct UserAccountRepository {
    pool: DbConnectionPool,
}

impl UserAccountRepository {
    pub fn new(pool: DbConnectionPool) -> UserAccountRepository {
        UserAccountRepository { pool }
    }

    pub async fn insert_one(
        &self,
        user_account_row: &UserAccountRow,
    ) -> Result<(), RepositoryError> {
        execute!(
            self.pool,
            diesel::insert_into(user_account).values(user_account_row)
        )?;
        Ok(())
    }

    pub async fn find_one_by_id(
        &self,
        account_id: &str,
    ) -> Result<UserAccountRow, RepositoryError> {
        first!(self.pool, user_account.filter(id.eq(account_id)))
    }

    pub async fn find_many_by_id(
        &self,
        ids: &[String],
    ) -> Result<Vec<UserAccountRow>, RepositoryError> {
        load!(self.pool, user_account.filter(id.eq_any(ids)))
    }
}
