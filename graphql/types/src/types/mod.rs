// M1 speced API is moved to their own files
// Types defined here are prototype types and should be removed before M1 release to avoid confusion (for consumers and devs)
pub mod name;
pub use self::name::*;

pub mod item;
pub use self::item::*;

pub mod item_stats;
pub use self::item_stats::*;

pub mod requisition;
pub use self::requisition::*;

pub mod requisition_line;
pub use self::requisition_line::*;

pub mod stock_line;
pub use self::stock_line::*;

pub mod location;
pub use self::location::*;

pub mod master_list;
pub use self::master_list::*;

pub mod master_list_line;
pub use self::master_list_line::*;

pub mod invoice_query;
pub use self::invoice_query::*;

pub mod invoice_line;
pub use self::invoice_line::*;

pub mod item_chart;
pub use self::item_chart::*;

pub mod store;
pub use self::store::*;

pub mod stocktake;
pub use self::stocktake::*;

pub mod stocktake_line;
pub use self::stocktake_line::*;

pub mod user;
pub use self::user::*;

use async_graphql::*;
pub struct DeleteResponse(pub String);
#[Object]
impl DeleteResponse {
    pub async fn id(&self) -> &str {
        &self.0
    }
}
