use super::{DBBackendConnection, DBTransaction};

use crate::database::{repository::RepositoryError, schema::NameRow};

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};

pub enum TxHolder<'a> {
    OwnedTx(DBTransaction),
    SharedTx(&'a DBTransaction),
}

impl<'a> TxHolder<'a> {
    fn get_tx(&self) -> &DBTransaction {
        match self {
            TxHolder::OwnedTx(ref t) => &t,
            TxHolder::SharedTx(t) => t,
        }
    }
}

pub trait TransactionGetter {
    fn get(&self) -> Result<TxHolder, RepositoryError>;
}

#[derive(Clone)]
pub struct PoolTransactionGetter {
    pool: Pool<ConnectionManager<DBBackendConnection>>,
}

impl TransactionGetter for PoolTransactionGetter {
    fn get(&self) -> Result<TxHolder, RepositoryError> {
        let conn = self.pool.get().map_err(|_| RepositoryError::DBError {
            msg: "Failed to open Connection".to_string(),
        })?;
        Ok(TxHolder::OwnedTx(conn))
    }
}

pub struct ExistingTransactionGetter<'a> {
    transaction: &'a DBTransaction,
}

impl<'a> TransactionGetter for ExistingTransactionGetter<'a> {
    fn get(&self) -> Result<TxHolder, RepositoryError> {
        Ok(TxHolder::SharedTx(self.transaction))
    }
}

#[derive(Clone)]
pub struct NameRepositoryImp<T: TransactionGetter> {
    //pool: Option<Pool<ConnectionManager<DBBackendConnection>>>,
    getter: T,
}

pub type NameRepository = NameRepositoryImp<PoolTransactionGetter>;

pub fn new_name_repository(
    pool: Pool<ConnectionManager<DBBackendConnection>>,
) -> NameRepositoryImp<PoolTransactionGetter> {
    NameRepositoryImp {
        getter: PoolTransactionGetter { pool },
    }
}

pub fn new_tx_name_repository<'a>(
    tx: &'a DBTransaction,
) -> NameRepositoryImp<ExistingTransactionGetter<'a>> {
    NameRepositoryImp {
        getter: ExistingTransactionGetter { transaction: tx },
    }
}

impl<T: TransactionGetter> NameRepositoryImp<T> {
    pub async fn insert_one(&self, name_row: &NameRow) -> Result<(), RepositoryError> {
        use crate::database::schema::diesel_schema::name_table::dsl::*;
        let tx_holder = self.getter.get()?;
        diesel::insert_into(name_table)
            .values(name_row)
            .execute(tx_holder.get_tx())?;
        Ok(())
    }

    pub async fn find_one_by_id(&self, name_id: &str) -> Result<NameRow, RepositoryError> {
        use crate::database::schema::diesel_schema::name_table::dsl::*;
        let tx_holder = self.getter.get()?;
        let result = name_table
            .filter(id.eq(name_id))
            .first(tx_holder.get_tx())?;
        Ok(result)
    }
}
