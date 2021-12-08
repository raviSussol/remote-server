use async_graphql::*;
use chrono::Utc;
use service::dashboard::invoice_count::{
    InvoiceCountError, InvoiceCountService, InvoiceCountServiceTrait,
};

use crate::errors::StandardError;
use crate::schema::types::invoice_query::InvoiceNodeType;
use crate::ContextExt;

#[derive(SimpleObject)]
pub struct InvoiceCounts {
    created: InvoiceCountsSummary,
}

#[derive(SimpleObject)]
pub struct InvoiceCountsSummary {
    pub today: i64,
    pub this_week: i64,
}

#[derive(Union)]
pub enum InvoiceCountsResponse {
    Response(InvoiceCounts),
}

pub fn invoice_counts(
    ctx: &Context<'_>,
    invoice_type: InvoiceNodeType,
    timezone_offset: Option<i32>,
) -> Result<InvoiceCountsResponse, StandardError> {
    let service_provider = ctx.service_provider();
    let service_ctx = service_provider.context()?;
    let service = InvoiceCountService {};
    let result = service.invoice_count_created(
        &service_ctx,
        invoice_type.into(),
        Utc::now(),
        timezone_offset,
    );
    let created = match result {
        Ok(created) => created,
        Err(err) => match err {
            InvoiceCountError::RepositoryError(err) => return Err(err.into()),
            InvoiceCountError::BadTimezoneOffset => {
                return Err(StandardError::BadUserInput(
                    "Invalid timezone offset".to_string(),
                ));
            }
        },
    };

    Ok(InvoiceCountsResponse::Response(InvoiceCounts {
        created: InvoiceCountsSummary {
            today: created.today,
            this_week: created.this_week,
        },
    }))
}
