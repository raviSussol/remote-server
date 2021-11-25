use super::{ListError, ListResult};
use crate::SingleRecordError;
use domain::{
    location::{Location, LocationFilter, LocationSort},
    PaginationOption,
};

pub mod query;

pub trait LocationQueryServiceTrait {
    fn get_locations(
        &self,
        pagination: Option<PaginationOption>,
        filter: Option<LocationFilter>,
        sort: Option<LocationSort>,
    ) -> Result<ListResult<Location>, ListError>;

    fn get_location(&self, id: String) -> Result<Location, SingleRecordError>;
}
