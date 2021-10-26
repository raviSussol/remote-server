use super::StorageConnection;

use crate::database::repository::RepositoryError;
use crate::database::schema::diesel_schema::{
    document::dsl as document_dsl, document_head::dsl as document_head_dsl,
};
use crate::database::schema::{DocumentHeadRow, DocumentRow};

use chrono::NaiveDateTime;
use diesel::prelude::*;

pub struct AncestorDetails {
    id: String,
    parents: String,
    timestamp: NaiveDateTime,
}

pub struct DocumentRepository<'a> {
    connection: &'a StorageConnection,
}

impl<'a> DocumentRepository<'a> {
    pub fn new(connection: &'a StorageConnection) -> Self {
        DocumentRepository { connection }
    }

    /// Get a specific document version
    pub fn find_one_by_id(&self, document_id: &str) -> Result<DocumentRow, RepositoryError> {
        let result = document_dsl::document
            .filter(document_dsl::id.eq(document_id))
            .first(&self.connection.connection);
        result.map_err(|err| RepositoryError::from(err))
    }

    /// Get the latest version of a document
    pub fn find_one_by_name(&self, document_name: &str) -> Result<DocumentRow, RepositoryError> {
        let head: DocumentHeadRow = document_head_dsl::document_head
            .filter(document_head_dsl::id.eq(document_name))
            .first(&self.connection.connection)?;
        self.find_one_by_id(&head.document_id)
    }

    /// Gets all document versions
    pub fn find_many_by_name(
        &self,
        document_name: &str,
    ) -> Result<Vec<DocumentRow>, RepositoryError> {
        let result = document_dsl::document
            .filter(document_dsl::name.eq(document_name))
            .load(&self.connection.connection)?;
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
