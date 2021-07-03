use crate::database::repository::Repository;
use crate::database::schema::ItemLineRow;

#[derive(Clone)]
pub struct ItemLineRepository {
    pool: sqlx::PgPool,
}

impl Repository for ItemLineRepository {}

impl ItemLineRepository {
    pub fn new(pool: sqlx::PgPool) -> ItemLineRepository {
        ItemLineRepository { pool }
    }

    pub async fn insert_one(&self, item_line: &ItemLineRow) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO item_line (id, item_id, store_id, batch, quantity)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            item_line.id,
            item_line.item_id,
            item_line.store_id,
            item_line.batch,
            item_line.quantity
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_one_by_id(&self, id: &str) -> Result<ItemLineRow, sqlx::Error> {
        let item_line = sqlx::query_as!(
            ItemLineRow,
            r#"
            SELECT id, item_id, store_id, batch, quantity
            from item_line
            where id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(item_line)
    }
}