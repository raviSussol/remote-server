use super::StorageConnection;

use crate::database::repository::RepositoryError;
use crate::database::schema::diesel_schema::{
    document::dsl as document_dsl, document_head::dsl as document_head_dsl,
};
use crate::database::schema::{DocumentHeadRow, DocumentRow};
use crate::domain::document::Document;

use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::prelude::*;

pub struct AncestorDetails {
    id: String,
    parents: String,
    timestamp: NaiveDateTime,
}

pub struct DocumentRepository<'a> {
    connection: &'a StorageConnection,
}

fn document_from_row(row: DocumentRow) -> Result<Document, RepositoryError> {
    let parents: Vec<String> =
        serde_json::from_str(&row.parents).map_err(|err| RepositoryError::DBError {
            msg: "Invalid parents data".to_string(),
            extra: format!("{}", err),
        })?;
    let data: serde_json::Value =
        serde_json::from_str(&row.data).map_err(|err| RepositoryError::DBError {
            msg: "Invalid data".to_string(),
            extra: format!("{}", err),
        })?;
    Ok(Document {
        id: row.id,
        name: row.name,
        parents,
        author: row.author,
        timestamp: DateTime::<Utc>::from_utc(row.timestamp, Utc),
        type_: row.type_,
        data,
    })
}

fn make_head_id(name: &str, store: &str) -> String {
    format!("{}@{}", name, store)
}

impl<'a> DocumentRepository<'a> {
    pub fn new(connection: &'a StorageConnection) -> Self {
        DocumentRepository { connection }
    }

    /// Get a specific document version
    pub fn find_one_by_id(&self, document_id: &str) -> Result<Document, RepositoryError> {
        let row: DocumentRow = document_dsl::document
            .filter(document_dsl::id.eq(document_id))
            .first(&self.connection.connection)?;

        document_from_row(row)
    }

    /// Get the latest version of a document
    pub fn find_one_by_name(
        &self,
        document_name: &str,
        store: &str,
    ) -> Result<Document, RepositoryError> {
        let head: DocumentHeadRow = document_head_dsl::document_head
            .filter(document_head_dsl::id.eq(make_head_id(document_name, store)))
            .first(&self.connection.connection)?;
        self.find_one_by_id(&head.head)
    }

    /// Gets all document versions
    pub fn find_many_by_name(&self, document_name: &str) -> Result<Vec<Document>, RepositoryError> {
        let rows: Vec<DocumentRow> = document_dsl::document
            .filter(document_dsl::name.eq(document_name))
            .load(&self.connection.connection)?;
        let mut result = Vec::<Document>::new();
        for row in rows {
            result.push(document_from_row(row)?);
        }
        Ok(result)
    }

    /// Gets ancestor details for the full document history.
    pub fn ancestor_details(
        &self,
        document_name: &str,
    ) -> Result<Vec<AncestorDetails>, RepositoryError> {
        let result: Vec<(String, String, NaiveDateTime)> = document_dsl::document
            .filter(document_dsl::name.eq(document_name))
            .select((
                document_dsl::id,
                document_dsl::parents,
                document_dsl::timestamp,
            ))
            .load(&self.connection.connection)?;
        Ok(result
            .into_iter()
            .map(|row| AncestorDetails {
                id: row.0,
                parents: row.1,
                timestamp: row.2,
            })
            .collect())
    }
}
