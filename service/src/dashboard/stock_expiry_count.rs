use chrono::NaiveDate;
use repository::{DateFilter, RepositoryError, StockLineFilter, StockLineRepository};

use crate::service_provider::ServiceContext;

pub trait StockExpiryCountServiceTrait: Send + Sync {
    /// # Arguments
    ///
    /// * date_time date at which the expired stock is counted
    fn count_expired_stock(
        &self,
        ctx: &ServiceContext,
        date_time: NaiveDate,
    ) -> Result<i64, RepositoryError> {
        StockExpiryServiceCount {}.count_expired_stock(ctx, date_time)
    }
}

pub struct StockExpiryServiceCount {}

impl StockExpiryCountServiceTrait for StockExpiryServiceCount {
    fn count_expired_stock(
        &self,
        ctx: &ServiceContext,
        date_time: NaiveDate,
    ) -> Result<i64, RepositoryError> {
        let repo = StockLineRepository::new(&ctx.connection);
        repo.count(Some(StockLineFilter::new().expiry_date(DateFilter {
            equal_to: None,
            before_or_equal_to: Some(date_time),
            after_or_equal_to: None,
        })))
    }
}
