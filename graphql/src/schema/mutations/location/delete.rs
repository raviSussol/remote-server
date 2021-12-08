use async_graphql::*;
use domain::location::DeleteLocation;
use repository::RepositoryError;
use service::location::delete::{
    DeleteLocationError as InError, LocationInUse as ServiceLocationInUse,
};

use crate::{
    errors::StandardError,
    schema::{
        mutations::{error::DatabaseError, DeleteResponse, RecordBelongsToAnotherStore},
        types::{Connector, InvoiceLineNode, RecordNotFound, StockLineNode},
    },
    ContextExt,
};

pub fn delete_location(
    ctx: &Context<'_>,
    input: DeleteLocationInput,
) -> Result<DeleteLocationResponse, StandardError> {
    use DeleteLocationResponse::*;
    let service_provider = ctx.service_provider();
    let service_context = service_provider.context()?;

    let result = match service_provider
        .location_service
        .delete_location(&service_context, input.into())
    {
        Ok(location_id) => Response(DeleteResponse(location_id)),
        Err(error) => DeleteLocationResponse::StructuredError(DeleteLocationError {
            error: map_error(error)?,
        }),
    };

    Ok(result)
}

pub fn map_error(error: InError) -> Result<DeleteLocationErrorInterface, StandardError> {
    use DeleteLocationErrorInterface as OutError;
    let standard_error = format!("{:#?}", error);
    let standard_error = match error {
        InError::LocationInUse(ServiceLocationInUse {
            stock_lines,
            invoice_lines,
        }) => {
            return Ok(OutError::LocationInUse(LocationInUse {
                stock_lines: stock_lines.into(),
                invoice_lines: invoice_lines.into(),
            }))
        }
        InError::LocationDoesNotExist => StandardError::BadUserInput(standard_error),
        InError::LocationDoesNotBelongToCurrentStore => StandardError::Forbidden(standard_error),
        InError::DatabaseError(_) => StandardError::InternalError(standard_error),
    };
    Err(standard_error)
}

#[derive(InputObject)]
pub struct DeleteLocationInput {
    pub id: String,
}

impl From<DeleteLocationInput> for DeleteLocation {
    fn from(DeleteLocationInput { id }: DeleteLocationInput) -> Self {
        DeleteLocation { id }
    }
}

#[derive(SimpleObject)]
pub struct DeleteLocationError {
    pub error: DeleteLocationErrorInterface,
}

#[derive(Union)]
pub enum DeleteLocationResponse {
    StructuredError(DeleteLocationError),
    Response(DeleteResponse),
}

#[derive(Interface)]
#[graphql(field(name = "description", type = "String"))]
pub enum DeleteLocationErrorInterface {
    LocationNotFound(RecordNotFound),
    RecordBelongsToAnotherStore(RecordBelongsToAnotherStore),
    LocationInUse(LocationInUse),
    DatabaseError(DatabaseError),
}

pub struct LocationInUse {
    stock_lines: Connector<StockLineNode>,
    invoice_lines: Connector<InvoiceLineNode>,
}

#[Object]
impl LocationInUse {
    pub async fn description(&self) -> &'static str {
        "Location in use"
    }

    pub async fn stock_lines(&self) -> &Connector<StockLineNode> {
        &self.stock_lines
    }

    pub async fn invoice_lines(&self) -> &Connector<InvoiceLineNode> {
        &self.invoice_lines
    }
}

impl From<RepositoryError> for DeleteLocationError {
    fn from(error: RepositoryError) -> Self {
        let error = DeleteLocationErrorInterface::DatabaseError(DatabaseError(error));
        DeleteLocationError { error }
    }
}
