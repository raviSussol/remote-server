use crate::database::schema::{
    ItemLineRow, ItemRow, NameRow, RequisitionLineRow, RequisitionRow, StoreRow, TransactLineRow,
    TransactRow,
};
use crate::database::DatabaseConnection;

#[derive(Clone)]
pub struct DataLoader {
    database: DatabaseConnection,
}

impl DataLoader {
    pub fn new(database: DatabaseConnection) -> DataLoader {
        DataLoader { database }
    }

    #[allow(dead_code)]
    pub async fn create_store(&self, store: &StoreRow) -> Result<(), sqlx::Error> {
        self.database.create_store(store).await
    }

    #[allow(dead_code)]
    pub async fn create_stores(&self, stores: &[StoreRow]) -> Result<(), sqlx::Error> {
        self.database.create_stores(stores).await
    }

    #[allow(dead_code)]
    pub async fn create_name(&self, name: &NameRow) -> Result<(), sqlx::Error> {
        self.database.create_name(name).await
    }

    #[allow(dead_code)]
    pub async fn create_names(&self, names: &[NameRow]) -> Result<(), sqlx::Error> {
        self.database.create_names(names).await
    }

    pub async fn create_item(&self, item: &ItemRow) -> Result<(), sqlx::Error> {
        self.database.create_item(item).await
    }

    pub async fn create_items(&self, items: &[ItemRow]) -> Result<(), sqlx::Error> {
        self.database.create_items(items).await
    }

    #[allow(dead_code)]
    pub async fn create_item_line(&self, item_line: &ItemLineRow) -> Result<(), sqlx::Error> {
        self.database.create_item_line(item_line).await
    }

    #[allow(dead_code)]
    pub async fn create_item_lines(&self, item_lines: &[ItemLineRow]) -> Result<(), sqlx::Error> {
        self.database.create_item_lines(item_lines).await
    }

    pub async fn create_requisition(
        &self,
        requisition: &RequisitionRow,
    ) -> Result<(), sqlx::Error> {
        self.database.create_requisition(requisition).await
    }

    pub async fn create_requisitions(
        &self,
        requisitions: &[RequisitionRow],
    ) -> Result<(), sqlx::Error> {
        self.database.create_requisitions(requisitions).await
    }

    pub async fn create_requisition_line(
        &self,
        requisition_line: &RequisitionLineRow,
    ) -> Result<(), sqlx::Error> {
        self.database
            .create_requisition_line(requisition_line)
            .await
    }

    pub async fn create_requisition_lines(
        &self,
        requisition_lines: &[RequisitionLineRow],
    ) -> Result<(), sqlx::Error> {
        self.database
            .create_requisition_lines(requisition_lines)
            .await
    }

    pub async fn get_store(&self, id: &str) -> Result<StoreRow, sqlx::Error> {
        self.database.get_store(id).await
    }

    pub async fn get_name(&self, id: &str) -> Result<NameRow, sqlx::Error> {
        self.database.get_name(id).await
    }

    pub async fn get_item(&self, id: &str) -> Result<ItemRow, sqlx::Error> {
        self.database.get_item(id).await
    }

    pub async fn get_item_line(&self, id: &str) -> Result<ItemLineRow, sqlx::Error> {
        self.database.get_item_line(id).await
    }

    pub async fn get_requisition(&self, id: &str) -> Result<RequisitionRow, sqlx::Error> {
        self.database.get_requisition(id).await
    }

    #[allow(dead_code)]
    pub async fn get_requisition_line(&self, id: &str) -> Result<RequisitionLineRow, sqlx::Error> {
        self.database.get_requisition_line(id).await
    }

    pub async fn get_requisition_lines(
        &self,
        requisition_id: &str,
    ) -> Result<Vec<RequisitionLineRow>, sqlx::Error> {
        self.database.get_requisition_lines(requisition_id).await
    }

    #[allow(dead_code)]
    pub async fn get_transact(&self, id: &str) -> Result<TransactRow, sqlx::Error> {
        self.database.get_transact(id).await
    }

    #[allow(dead_code)]
    pub async fn get_transacts(&self, name_id: &str) -> Result<Vec<TransactRow>, sqlx::Error> {
        self.database.get_transacts(name_id).await
    }

    #[allow(dead_code)]
    pub async fn get_transact_line(&self, id: &str) -> Result<TransactLineRow, sqlx::Error> {
        self.database.get_transact_line(id).await
    }

    #[allow(dead_code)]
    pub async fn get_transact_lines(
        &self,
        transact_id: &str,
    ) -> Result<Vec<TransactLineRow>, sqlx::Error> {
        self.database.get_transact_lines(transact_id).await
    }
}
