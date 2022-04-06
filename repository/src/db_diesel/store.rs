use crate::diesel_macros::{apply_equal_filter, apply_sort_no_case};
use crate::schema::NameRow;
use crate::{EqualFilter, Pagination, SimpleStringFilter, Sort};

use crate::{
    diesel_macros::apply_simple_string_filter,
    schema::{
        diesel_schema::{name, name::dsl as name_dsl},
        store::{store, store::dsl as store_dsl},
        StoreRow,
    },
    DBType, RepositoryError, StorageConnection,
};

use diesel::dsl::InnerJoin;
use diesel::{dsl::IntoBoxed, prelude::*};

#[derive(Debug, PartialEq, Clone)]
pub struct Store {
    pub store_row: StoreRow,
    pub name_row: NameRow,
}

#[derive(Debug, Clone, Default)]
pub struct StoreFilter {
    pub id: Option<EqualFilter<String>>,
    pub code: Option<SimpleStringFilter>,
    pub name: Option<SimpleStringFilter>,
    pub name_code: Option<SimpleStringFilter>,
    pub site_id: Option<EqualFilter<i32>>,
}

#[derive(PartialEq, Debug)]
pub enum StoreSortField {
    Code,
    Name,
    NameCode,
}

pub type StoreSort = Sort<StoreSortField>;

pub type StoreJoin = (StoreRow, NameRow);

impl StoreFilter {
    pub fn new() -> StoreFilter {
        StoreFilter::default()
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

    pub fn site_id(mut self, filter: EqualFilter<i32>) -> Self {
        self.site_id = Some(filter);
        self
    }
}

pub struct StoreRepository<'a> {
    connection: &'a StorageConnection,
}

impl<'a> StoreRepository<'a> {
    pub fn new(connection: &'a StorageConnection) -> Self {
        StoreRepository { connection }
    }

    pub fn count(&self, filter: Option<StoreFilter>) -> Result<i64, RepositoryError> {
        let query = create_filtered_query(filter);
        Ok(query.count().get_result(&self.connection.connection)?)
    }

    pub fn query_one(&self, filter: StoreFilter) -> Result<Option<Store>, RepositoryError> {
        Ok(self.query_by_filter(filter)?.pop())
    }

    pub fn query_by_filter(&self, filter: StoreFilter) -> Result<Vec<Store>, RepositoryError> {
        self.query(Pagination::new(), Some(filter), None)
    }

    pub fn query(
        &self,
        pagination: Pagination,
        filter: Option<StoreFilter>,
        sort: Option<StoreSort>,
    ) -> Result<Vec<Store>, RepositoryError> {
        // TODO (beyond M1), check that store_id matches current store
        let mut query = create_filtered_query(filter);
        if let Some(sort) = sort {
            match sort.key {
                StoreSortField::Code => {
                    apply_sort_no_case!(query, sort, store_dsl::code);
                }
                StoreSortField::Name => {
                    apply_sort_no_case!(query, sort, name_dsl::name_);
                }
                StoreSortField::NameCode => {
                    apply_sort_no_case!(query, sort, name_dsl::code);
                }
            }
        } else {
            query = query.order(store_dsl::id.asc())
        }
        let result = query
            .offset(pagination.offset as i64)
            .limit(pagination.limit as i64)
            .load::<StoreJoin>(&self.connection.connection)?;

        Ok(result.into_iter().map(to_domain).collect())
    }
}

type BoxedStoreQuery = IntoBoxed<'static, InnerJoin<store::table, name::table>, DBType>;

fn create_filtered_query(filter: Option<StoreFilter>) -> BoxedStoreQuery {
    let mut query = store_dsl::store.inner_join(name_dsl::name).into_boxed();

    if let Some(f) = filter {
        let StoreFilter {
            id,
            code,
            name,
            name_code,
            site_id,
        } = f;

        apply_equal_filter!(query, id, store_dsl::id);
        apply_simple_string_filter!(query, code, store_dsl::code);
        apply_simple_string_filter!(query, name, name_dsl::name_);
        apply_simple_string_filter!(query, name_code, name_dsl::code);
        apply_equal_filter!(query, site_id, store_dsl::site_id);
    }

    query
}

fn to_domain((store_row, name_row): StoreJoin) -> Store {
    Store {
        store_row,
        name_row,
    }
}
