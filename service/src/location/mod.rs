use super::{ListError, ListResult};
use crate::SingleRecordError;
use domain::{
    location::{Location, LocationFilter, LocationSort},
    PaginationOption,
};
use repository::StorageConnection;

pub mod query;

pub trait LocationServiceQuery: Sync + Send {
    fn get_locations(
        &self,
        connection: &StorageConnection,
        pagination: Option<PaginationOption>,
        filter: Option<LocationFilter>,
        sort: Option<LocationSort>,
    ) -> Result<ListResult<Location>, ListError>;

    fn get_location(
        &self,
        connection: &StorageConnection,
        id: String,
    ) -> Result<Location, SingleRecordError>;
}

pub struct LocationService;
