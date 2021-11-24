use domain::{
    location::{Location, LocationFilter, LocationSort},
    Pagination, PaginationOption,
};
use repository::{LocationRepository, StorageConnection};

use crate::{
    get_default_pagination, i64_to_u32, service_provider::Services, ListError, ListResult,
    SingleRecordError,
};

use super::{LocationService, LocationServiceQuery};

pub const MAX_LIMIT: u32 = 1000;
pub const MIN_LIMIT: u32 = 1;

impl LocationServiceQuery for LocationService {
    fn get_locations(
        &self,
        connection: &StorageConnection,
        pagination: Option<PaginationOption>,
        filter: Option<LocationFilter>,
        sort: Option<LocationSort>,
    ) -> Result<ListResult<Location>, ListError> {
        let pagination = get_default_pagination(pagination, MAX_LIMIT, MIN_LIMIT)?;
        let repository = LocationRepository::new(connection);

        Ok(ListResult {
            rows: repository.query(pagination, filter.clone(), sort)?,
            count: i64_to_u32(repository.count(filter)?),
        })
    }

    fn get_location(
        &self,
        connection: &StorageConnection,
        id: String,
    ) -> Result<Location, SingleRecordError> {
        let repository = LocationRepository::new(connection);

        let mut result = repository.query(
            Pagination::one(),
            Some(LocationFilter::new().match_id(&id)),
            None,
        )?;

        if let Some(record) = result.pop() {
            Ok(record)
        } else {
            Err(SingleRecordError::NotFound(id))
        }
    }
}

// Add to Services
impl<'a> Services<'a> {
    pub fn get_locations(
        &self,
        pagination: Option<PaginationOption>,
        filter: Option<LocationFilter>,
        sort: Option<LocationSort>,
    ) -> Result<ListResult<Location>, ListError> {
        self.services_instances.location_service.get_locations(
            self.get_connection(),
            pagination,
            filter,
            sort,
        )
    }

    pub fn get_location(&self, id: String) -> Result<Location, SingleRecordError> {
        self.services_instances
            .location_service
            .get_location(self.get_connection(), id)
    }
}

#[cfg(test)]
mod tests;
