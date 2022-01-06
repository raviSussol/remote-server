use async_graphql::*;
use service::invoice_line::{
    DeleteOutboundShipmentUnallocatedLine as ServiceInput,
    DeleteOutboundShipmentUnallocatedLineError as ServiceError,
};

use crate::{
    schema::mutations::{DeleteResponse, RecordDoesNotExist},
    standard_graphql_error::StandardGraphqlError,
    ContextExt,
};

#[derive(InputObject)]
#[graphql(name = "DeleteOutboundShipmentUnallocatedLineInput")]
pub struct Input {
    pub id: String,
}

#[derive(Interface)]
#[graphql(name = "DeleteOutboundShipmentUnallocatedLineErrorInterface")]
#[graphql(field(name = "description", type = "String"))]
pub enum ErrorInterface {
    RecordDoesNotExist(RecordDoesNotExist),
}

#[derive(SimpleObject)]
#[graphql(name = "DeleteOutboundShipmentUnallocatedLineError")]
pub struct Error {
    pub error: ErrorInterface,
}

#[derive(Union)]
#[graphql(name = "DeleteOutboundShipmentUnallocatedLineResponse")]
pub enum Response {
    Error(Error),
    Response(DeleteResponse),
}

impl From<Input> for ServiceInput {
    fn from(Input { id }: Input) -> Self {
        ServiceInput { id }
    }
}

pub fn op(ctx: &Context<'_>, input: Input) -> Result<Response> {
    let service_provider = ctx.service_provider();
    let service_context = service_provider.context()?;

    let response = match service_provider
        .outbound_shipment_line
        .delete_outbound_shipment_unallocated_line(&service_context, input.into())
    {
        Ok(id) => Response::Response(DeleteResponse(id)),
        Err(error) => Response::Error(Error {
            error: map_error(error)?,
        }),
    };

    Ok(response)
}

fn map_error(error: ServiceError) -> Result<ErrorInterface> {
    use StandardGraphqlError::*;
    let formatted_error = format!("{:#?}", error);

    let graphql_error = match error {
        // Structured Errors
        ServiceError::LineDoesNotExist => {
            return Ok(ErrorInterface::RecordDoesNotExist(RecordDoesNotExist {}))
        }
        // Standard Graphql Errors
        ServiceError::LineIsNotUnallocatedLine => BadUserInput(formatted_error),
        ServiceError::DatabaseError(_) => InternalError(formatted_error),
    };

    Err(graphql_error.extend())
}
