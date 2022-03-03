use super::{DBType, StorageConnection};
use crate::{
    diesel_macros::{apply_equal_filter, apply_simple_string_filter, apply_sort_no_case},
    repository_error::RepositoryError,
    schema::{
        diesel_schema::{
            name, name::dsl as name_dsl, name_store_join,
            name_store_join::dsl as name_store_join_dsl, store, store::dsl as store_dsl,
        },
        NameRow, NameStoreJoinRow, StoreRow,
    },
};
use crate::{EqualFilter, Pagination, SimpleStringFilter, Sort};

use diesel::{
    dsl::{IntoBoxed, LeftJoin},
    prelude::*,
};
use util::constants::SYSTEM_NAME_CODES;

#[derive(PartialEq, Debug, Clone, Default)]
pub struct Name {
    pub name_row: NameRow,
    pub name_store_join_row: Option<NameStoreJoinRow>,
    pub store_row: Option<StoreRow>,
}

#[derive(Clone, Default)]
pub struct NameFilter {
    pub id: Option<EqualFilter<String>>,
    pub name: Option<SimpleStringFilter>,
    pub code: Option<SimpleStringFilter>,
    pub is_customer: Option<bool>,
    pub is_supplier: Option<bool>,
    pub store_id: Option<EqualFilter<String>>,
    pub is_store: Option<bool>,
    pub store_code: Option<SimpleStringFilter>,
}

pub enum NameSortField {
    Name,
    Code,
}

pub type NameSort = Sort<NameSortField>;

type NameAndNameStoreJoin = (NameRow, Option<NameStoreJoinRow>, Option<StoreRow>);

pub struct NameQueryRepository<'a> {
    connection: &'a StorageConnection,
}

impl<'a> NameQueryRepository<'a> {
    pub fn new(connection: &'a StorageConnection) -> Self {
        NameQueryRepository { connection }
    }

    pub fn count(
        &self,
        store_id: &str,
        filter: Option<NameFilter>,
    ) -> Result<i64, RepositoryError> {
        // TODO (beyond M1), check that store_id matches current store
        let query = create_filtered_query(store_id, filter);

        Ok(query.count().get_result(&self.connection.connection)?)
    }

    pub fn query_by_filter(
        &self,
        store_id: &str,
        filter: NameFilter,
    ) -> Result<Vec<Name>, RepositoryError> {
        self.query(store_id, Pagination::new(), Some(filter), None)
    }

    pub fn query_one(
        &self,
        store_id: &str,
        filter: NameFilter,
    ) -> Result<Option<Name>, RepositoryError> {
        Ok(self.query_by_filter(store_id, filter)?.pop())
    }

    pub fn query(
        &self,
        store_id: &str,
        pagination: Pagination,
        filter: Option<NameFilter>,
        sort: Option<NameSort>,
    ) -> Result<Vec<Name>, RepositoryError> {
        // TODO (beyond M1), check that store_id matches current store
        let mut query = create_filtered_query(store_id, filter);

        if let Some(sort) = sort {
            match sort.key {
                NameSortField::Name => {
                    apply_sort_no_case!(query, sort, name_dsl::name_);
                }
                NameSortField::Code => {
                    apply_sort_no_case!(query, sort, name_dsl::code);
                }
            }
        } else {
            query = query.order(name_dsl::id.asc())
        }

        let final_query = query
            .offset(pagination.offset as i64)
            .limit(pagination.limit as i64);

        // Debug diesel query
        println!(
            "{}",
            diesel::debug_query::<DBType, _>(&final_query).to_string()
        );

        let result = final_query.load::<NameAndNameStoreJoin>(&self.connection.connection)?;

        Ok(result.into_iter().map(to_domain).collect())
    }
}

fn to_domain((name_row, name_store_join_row, store_row): NameAndNameStoreJoin) -> Name {
    Name {
        name_row,
        name_store_join_row,
        store_row,
    }
}

type BoxedNameQuery = IntoBoxed<
    'static,
    LeftJoin<LeftJoin<name::table, name_store_join::table>, store::table>,
    DBType,
>;

fn apply_special_filters(store_id: &str, mut query: BoxedNameQuery) -> BoxedNameQuery {
    // Filter out special name and current store
    query = query.filter(name::code.ne_all(SYSTEM_NAME_CODES));
    // Only one instance of name store join
    // query = query.filter(
    //     name_store_join_dsl::store_id
    //         .eq(store_id.to_string())
    //         .or(name_store_join_dsl::store_id.is_null()),
    // );

    query
}

fn create_filtered_query(store_id: &str, filter: Option<NameFilter>) -> BoxedNameQuery {
    let query = name_dsl::name
        .left_join(
            name_store_join_dsl::name_store_join.on(name_store_join_dsl::name_id
                .eq(name_dsl::id)
                .and(name_store_join_dsl::store_id.eq(store_id))),
        )
        .left_join(store_dsl::store)
        .into_boxed();

    let mut query = apply_special_filters(store_id, query);

    if let Some(f) = filter {
        let NameFilter {
            id,
            name,
            code,
            is_customer,
            is_supplier,
            store_id,
            is_store,
            store_code,
        } = f;

        apply_equal_filter!(query, id, name_dsl::id);
        apply_simple_string_filter!(query, code, name_dsl::code);
        apply_simple_string_filter!(query, name, name_dsl::name_);
        apply_simple_string_filter!(query, store_code, store_dsl::code);
        apply_equal_filter!(query, store_id, store_dsl::id);

        if let Some(is_customer) = is_customer {
            query = query.filter(name_store_join_dsl::name_is_customer.eq(is_customer));
        }
        if let Some(is_supplier) = is_supplier {
            query = query.filter(name_store_join_dsl::name_is_supplier.eq(is_supplier));
        }

        query = match is_store {
            Some(true) => query.filter(store_dsl::id.is_not_null()),
            Some(false) => query.filter(store_dsl::id.is_null()),
            None => query,
        }
    }

    query
}

impl NameFilter {
    pub fn new() -> NameFilter {
        NameFilter::default()
    }

    pub fn id(mut self, filter: EqualFilter<String>) -> Self {
        self.id = Some(filter);
        self
    }

    pub fn code(mut self, filter: SimpleStringFilter) -> Self {
        self.code = Some(filter);
        self
    }

    pub fn name(mut self, filter: SimpleStringFilter) -> Self {
        self.name = Some(filter);
        self
    }

    pub fn match_is_supplier(mut self, value: bool) -> Self {
        self.is_supplier = Some(value);
        self
    }

    pub fn store_id(mut self, filter: EqualFilter<String>) -> Self {
        self.store_id = Some(filter);
        self
    }
}

impl Name {
    pub fn is_customer(&self) -> bool {
        self.name_store_join_row
            .as_ref()
            .map(|name_store_join_row| name_store_join_row.name_is_customer)
            .unwrap_or(false)
    }

    pub fn is_supplier(&self) -> bool {
        self.name_store_join_row
            .as_ref()
            .map(|name_store_join_row| name_store_join_row.name_is_supplier)
            .unwrap_or(false)
    }

    pub fn store_id(&self) -> Option<&str> {
        self.store_row
            .as_ref()
            .map(|store_row| store_row.id.as_str())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        test_db, Pagination, DEFAULT_PAGINATION_LIMIT,
        {
            db_diesel::{NameQueryRepository, NameRepository},
            mock::MockDataInserts,
            schema::NameRow,
        },
    };

    use std::convert::TryFrom;

    use super::{Name, NameSort, NameSortField};

    fn data() -> (Vec<NameRow>, Vec<Name>) {
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

            queries.push(Name {
                name_row: NameRow {
                    id: format!("id{:05}", index),
                    name: format!("name{}", index),
                    code: format!("code{}", index),

                    is_customer: true,
                    is_supplier: true,
                },
                name_store_join_row: None,
                store_row: None,
            });
        }
        (rows, queries)
    }

    #[actix_rt::test]
    async fn test_name_query_repository() {
        // Prepare
        let (_, storage_connection, _, _) =
            test_db::setup_all("test_name_query_repository", MockDataInserts::none()).await;
        let repository = NameQueryRepository::new(&storage_connection);

        let (rows, queries) = data();
        for row in rows {
            NameRepository::new(&storage_connection)
                .upsert_one(&row)
                .unwrap();
        }

        let default_page_size = usize::try_from(DEFAULT_PAGINATION_LIMIT).unwrap();
        let store_id = "store_a";

        // Test

        // .count()
        assert_eq!(
            usize::try_from(repository.count(store_id, None).unwrap()).unwrap(),
            queries.len()
        );

        // .query, no pagenation (default)
        assert_eq!(
            repository
                .query(store_id, Pagination::new(), None, None)
                .unwrap()
                .len(),
            default_page_size
        );

        // .query, pagenation (offset 10)
        let result = repository
            .query(
                store_id,
                Pagination {
                    offset: 10,
                    limit: DEFAULT_PAGINATION_LIMIT,
                },
                None,
                None,
            )
            .unwrap();
        assert_eq!(result.len(), default_page_size);
        assert_eq!(result[0], queries[10]);
        assert_eq!(
            result[default_page_size - 1],
            queries[10 + default_page_size - 1]
        );

        // .query, pagenation (first 10)
        let result = repository
            .query(
                store_id,
                Pagination {
                    offset: 0,
                    limit: 10,
                },
                None,
                None,
            )
            .unwrap();
        assert_eq!(result.len(), 10);
        assert_eq!(*result.last().unwrap(), queries[9]);

        // .query, pagenation (offset 150, first 90) <- more then records in table
        let result = repository
            .query(
                store_id,
                Pagination {
                    offset: 150,
                    limit: 90,
                },
                None,
                None,
            )
            .unwrap();
        assert_eq!(result.len(), queries.len() - 150);
        assert_eq!(result.last().unwrap(), queries.last().unwrap());
    }

    // TODO need to test name_store_join, but it also requires 'store' records to be add and name_store_join helpers
    // which i think might be too much for this test ? Ideally we would have a database snapshot to load in tests
    // I've tested locally with graphIQL, seems to work

    #[actix_rt::test]
    async fn test_name_query_sort() {
        let (_, connection, _, _) =
            test_db::setup_all("test_name_query_sort", MockDataInserts::all()).await;
        let repo = NameQueryRepository::new(&connection);

        let store_id = "store_a";
        let mut names = repo.query(store_id, Pagination::new(), None, None).unwrap();

        let sorted = repo
            .query(
                store_id,
                Pagination::new(),
                None,
                Some(NameSort {
                    key: NameSortField::Name,
                    desc: None,
                }),
            )
            .unwrap();

        names.sort_by(|a, b| {
            a.name_row
                .name
                .to_lowercase()
                .cmp(&b.name_row.name.to_lowercase())
        });

        for (count, name) in names.iter().enumerate() {
            assert_eq!(
                name.name_row.name.clone().to_lowercase(),
                sorted[count].name_row.name.clone().to_lowercase(),
            );
        }

        let sorted = repo
            .query(
                store_id,
                Pagination::new(),
                None,
                Some(NameSort {
                    key: NameSortField::Code,
                    desc: Some(true),
                }),
            )
            .unwrap();

        names.sort_by(|b, a| {
            a.name_row
                .code
                .to_lowercase()
                .cmp(&b.name_row.code.to_lowercase())
        });

        for (count, name) in names.iter().enumerate() {
            assert_eq!(
                name.name_row.code.clone().to_lowercase(),
                sorted[count].name_row.code.clone().to_lowercase(),
            );
        }
    }
}
