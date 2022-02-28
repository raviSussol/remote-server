mod central_sync_buffer;
mod changelog;
mod invoice;
mod invoice_line;
mod invoice_stats;
mod item;
mod item_is_visible;
mod item_stats;
mod key_value_store;
mod location;
mod master_list;
mod master_list_line;
mod master_list_name_join;
mod name;
mod name_store_join;
mod number;
mod remote_sync_buffer;
mod requisition;
mod requisition_line;
mod stock_line;
mod stocktake;
mod stocktake_line;
mod store;
mod sync_out;
mod unit;
mod user_account;

pub mod diesel_schema;

#[derive(Clone)]
pub enum DatabaseRow {
    Unit(UnitRow),
    Item(ItemRow),
    StockLine(StockLineRow),
    Name(NameRow),
    Requisition(RequisitionRow),
    RequisitionLine(RequisitionLineRow),
    Store(StoreRow),
    Invoice(InvoiceRow),
    InvoiceLine(InvoiceLineRow),
    UserAccount(UserAccountRow),
    SyncOut(SyncOutRow),
}

pub use central_sync_buffer::CentralSyncBufferRow;
pub use changelog::*;
pub use invoice::{InvoiceRow, InvoiceRowStatus, InvoiceRowType};
pub use invoice_line::{InvoiceLineRow, InvoiceLineRowType};
pub use invoice_stats::InvoiceStatsRow;
pub use item::{ItemRow, ItemRowType};
pub use item_is_visible::ItemIsVisibleRow;
pub use item_stats::*;
pub use key_value_store::*;
pub use location::LocationRow;
pub use master_list::*;
pub use master_list_line::*;
pub use master_list_name_join::MasterListNameJoinRow;
pub use name::NameRow;
pub use name_store_join::NameStoreJoinRow;
pub use number::{NumberRow, NumberRowType};
pub use remote_sync_buffer::*;
pub use requisition::*;
pub use requisition_line::RequisitionLineRow;
pub use stock_line::StockLineRow;
pub use stocktake::*;
pub use stocktake_line::*;
pub use store::StoreRow;
pub use sync_out::{SyncOutRow, SyncOutRowActionType, SyncOutRowTableNameType};
pub use unit::UnitRow;
pub use user_account::UserAccountRow;
