mod line;
mod service_line;
pub mod unallocated_line;

pub use line::*;
pub use service_line::*;

use async_graphql::*;

#[derive(InputObject)]
pub struct TaxUpdate {
    /// Set or unset the tax value (in percentage)
    pub percentage: Option<f64>,
}
