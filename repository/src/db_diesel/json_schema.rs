use super::StorageConnection;

use crate::schema::diesel_schema::json_schema::dsl as json_schema_dsl;
use crate::schema::JSONSchemaRow;
use crate::RepositoryError;

use diesel::prelude::*;
use domain::json_schema::JSONSchema;

pub struct JsonSchemaRepository<'a> {
    connection: &'a StorageConnection,
}

fn schema_from_row(schema_row: JSONSchemaRow) -> Result<JSONSchema, RepositoryError> {
    let parsed_schema: serde_json::Value =
        serde_json::from_str(&schema_row.schema).map_err(|err| RepositoryError::DBError {
            msg: "Invalid schema data".to_string(),
            extra: format!("{}", err),
        })?;
    Ok(JSONSchema {
        id: schema_row.id,
        schema: parsed_schema,
    })
}

fn row_from_schema(schema: &JSONSchema) -> Result<JSONSchemaRow, RepositoryError> {
    let data = serde_json::to_string(&schema.schema).map_err(|err| RepositoryError::DBError {
        msg: "Can't serialize data".to_string(),
        extra: format!("{}", err),
    })?;
    Ok(JSONSchemaRow {
        id: schema.id.to_owned(),
        schema: data,
    })
}

impl<'a> JsonSchemaRepository<'a> {
    pub fn new(connection: &'a StorageConnection) -> Self {
        JsonSchemaRepository { connection }
    }

    #[cfg(feature = "postgres")]
    pub fn upsert_one(&self, schema: &JSONSchema) -> Result<(), RepositoryError> {
        let row = row_from_schema(schema)?;
        diesel::insert_into(json_schema_dsl::json_schema)
            .values(row)
            .on_conflict(json_schema_dsl::id)
            .do_update()
            .set(row)
            .execute(&self.connection.connection)?;
        Ok(())
    }

    #[cfg(not(feature = "postgres"))]
    pub fn upsert_one(&self, schema: &JSONSchema) -> Result<(), RepositoryError> {
        let row = row_from_schema(schema)?;
        diesel::replace_into(json_schema_dsl::json_schema)
            .values(row)
            .execute(&self.connection.connection)?;
        Ok(())
    }

    /// Get a specific document version
    pub fn find_one_by_id(&self, document_id: &str) -> Result<JSONSchema, RepositoryError> {
        let row = json_schema_dsl::json_schema
            .filter(json_schema_dsl::id.eq(document_id))
            .first(&self.connection.connection)?;

        schema_from_row(row)
    }

    /// Gets all document versions
    pub fn find_many_by_ids(&self, ids: &[String]) -> Result<Vec<JSONSchema>, RepositoryError> {
        let rows: Vec<JSONSchemaRow> = json_schema_dsl::json_schema
            .filter(json_schema_dsl::id.eq_any(ids))
            .load(&self.connection.connection)?;
        let mut result = Vec::<JSONSchema>::new();
        for row in rows {
            result.push(schema_from_row(row)?);
        }
        Ok(result)
    }
}
