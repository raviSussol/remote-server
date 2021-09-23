use super::{get_connection, DBBackendConnection};
use crate::{
    database::{
        repository::RepositoryError,
        schema::{
            diesel_schema::{
                name_store_join::dsl::name_store_join, name_table::dsl as name_table_dsl,
            },
            NameRow, NameStoreJoinRow,
        },
    },
    server::service::graphql::schema::{
        queries::pagination::{Pagination, PaginationOption},
        types::NameQuery,
    },
};

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};

type NameAndNameStoreJoin = (NameRow, Option<NameStoreJoinRow>);

impl From<NameAndNameStoreJoin> for NameQuery {
    fn from((name_row, name_store_join_row_option): NameAndNameStoreJoin) -> Self {
        let (is_customer, is_supplier) = match name_store_join_row_option {
            Some(name_store_join_row) => (
                name_store_join_row.name_is_customer,
                name_store_join_row.name_is_supplier,
            ),
            None => (false, false),
        };

        NameQuery {
            id: name_row.id,
            name: name_row.name,
            code: name_row.code,
            is_customer,
            is_supplier,
        }
    }
}

pub struct NameQueryRepository {
    pool: Pool<ConnectionManager<DBBackendConnection>>,
}

impl NameQueryRepository {
    pub fn new(pool: Pool<ConnectionManager<DBBackendConnection>>) -> NameQueryRepository {
        NameQueryRepository { pool }
    }

    pub fn count(&self) -> Result<i64, RepositoryError> {
        let connection = get_connection(&self.pool)?;
        Ok(name_table_dsl::name_table
            .count()
            .get_result(&*connection)?)
    }

    pub fn all(&self, pagination: &Option<Pagination>) -> Result<Vec<NameQuery>, RepositoryError> {
        let connection = get_connection(&self.pool)?;
        // TODO (beyond M1), check that store_id matches current store
        Ok(name_table_dsl::name_table
            .left_join(name_store_join)
            .offset(pagination.offset())
            .limit(pagination.first())
            .order(name_table_dsl::id.asc())
            .load::<NameAndNameStoreJoin>(&*connection)?
            .into_iter()
            .map(NameQuery::from)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        database::{
            repository::{NameQueryRepository, NameRepository},
            schema::NameRow,
        },
        server::service::graphql::schema::{
            queries::pagination::{Pagination, DEFAULT_PAGE_SIZE},
            types::NameQuery,
        },
        util::test_db,
    };
    use std::convert::TryFrom;

    fn data() -> (Vec<NameRow>, Vec<NameQuery>) {
        let mut rows = Vec::new();
        let mut queries = Vec::new();
        for index in 0..200 {
            rows.push(NameRow {
                id: format!("id{:05}", index),
                name: format!("name{}", index),
                code: format!("code{}", index),
                is_customer: true,
                is_supplier: true,
            });

            queries.push(NameQuery {
                id: format!("id{:05}", index),
                name: format!("name{}", index),
                code: format!("code{}", index),
                is_customer: false,
                is_supplier: false,
            });
        }
        (rows, queries)
    }

    #[actix_rt::test]
    async fn test_name_query_repository() {
        // Prepare
        let (pool, _, connection) = test_db::setup_all("test_name_query_repository", false).await;
        let repository = NameQueryRepository::new(pool.clone());

        let (rows, queries) = data();
        for row in rows {
            NameRepository::upsert_one_tx(&connection, &row).unwrap();
        }

        let default_page_size = usize::try_from(DEFAULT_PAGE_SIZE).unwrap();

        // Test

        // .count()
        assert_eq!(
            usize::try_from(repository.count().unwrap()).unwrap(),
            queries.len()
        );

        // .all, no pagenation (default)
        assert_eq!(repository.all(&None).unwrap().len(), default_page_size);

        // .all, pagenation (offset 10)
        let result = repository
            .all(&Some(Pagination {
                offset: Some(10),
                first: None,
            }))
            .unwrap();
        assert_eq!(result.len(), default_page_size);
        assert_eq!(result[0], queries[10]);
        assert_eq!(
            result[default_page_size - 1],
            queries[10 + default_page_size - 1]
        );

        // .all, pagenation (first 10)
        let result = repository
            .all(&Some(Pagination {
                offset: None,
                first: Some(10),
            }))
            .unwrap();
        assert_eq!(result.len(), 10);
        assert_eq!(*result.last().unwrap(), queries[9]);

        // .all, pagenation (offset 150, first 90) <- more then records in table
        let result = repository
            .all(&Some(Pagination {
                offset: Some(150),
                first: Some(90),
            }))
            .unwrap();
        assert_eq!(result.len(), queries.len() - 150);
        assert_eq!(result.last().unwrap(), queries.last().unwrap());
    }

    // TODO need to test name_store_join, but it also requires 'store' records to be add and name_store_join helpers
    // which i think might be too much for this test ? Ideally we would have a database snapshot to load in tests
    // I've tested locally with graphIQL, seems to work
}