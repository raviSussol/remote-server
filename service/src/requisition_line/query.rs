use domain::PaginationOption;
use repository::{RequisitionLine, RequisitionLineFilter, RequisitionLineRepository};

use crate::{
    get_default_pagination, i64_to_u32, service_provider::ServiceContext, ListError, ListResult,
};

pub const MAX_LIMIT: u32 = 2000;
pub const MIN_LIMIT: u32 = 1;

pub fn get_requisition_lines(
    ctx: &ServiceContext,
    pagination: Option<PaginationOption>,
    filter: Option<RequisitionLineFilter>,
) -> Result<ListResult<RequisitionLine>, ListError> {
    let pagination = get_default_pagination(pagination, MAX_LIMIT, MIN_LIMIT)?;
    let repository = RequisitionLineRepository::new(&ctx.connection);
    Ok(ListResult {
        rows: repository.query(pagination, filter.clone())?,
        count: i64_to_u32(repository.count(filter)?),
    })
}

#[cfg(test)]
mod test {
    use domain::EqualFilter;
    use repository::{mock::{MockDataInserts, mock_draft_request_requisition_line}, test_db::setup_all, RequisitionLineFilter};

    use crate::service_provider::ServiceProvider;

    #[actix_rt::test]
    async fn requistion_line_service_queries() {
        let (_, _, connection_manager, _) =
            setup_all("test_requisition_line_filter", MockDataInserts::all()).await;

        let service_provider = ServiceProvider::new(connection_manager);
        let context = service_provider.context().unwrap();
        let service = service_provider.requisition_line_service;

        // RequisitionLines
        let result = service
            .get_requisition_lines(
                &context,
                None,
                Some(
                    RequisitionLineFilter::new()
                        .id(EqualFilter::equal_to(
                            &mock_draft_request_requisition_line().id,
                        ))
                        .requisition_id(EqualFilter::equal_to(
                            &mock_draft_request_requisition_line().requisition_id,
                        )),
                ),
            )
            .unwrap();

        assert_eq!(result.count, 1);
        assert_eq!(
            result.rows[0].requisition_line_row.id,
            mock_draft_request_requisition_line().id
        );
    }
}
