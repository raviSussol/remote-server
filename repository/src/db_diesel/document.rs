use super::StorageConnection;

use crate::schema::diesel_schema::{
    document::dsl as document_dsl, document_head::dsl as document_head_dsl,
};
use crate::schema::{DocumentHeadRow, DocumentRow};
use crate::RepositoryError;

use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::prelude::*;
use domain::document::{AncestorDetail, Document};

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

    let document = Document {
        id: row.id,
        name: row.name,
        parents,
        author: row.author,
        timestamp: DateTime::<Utc>::from_utc(row.timestamp, Utc),
        type_: row.type_,
        data,
        schema_id: row.schema_id,
    };

    Ok(document)
}

fn row_from_document(doc: &Document) -> Result<DocumentRow, RepositoryError> {
    let parents = serde_json::to_string(&doc.parents).map_err(|err| RepositoryError::DBError {
        msg: "Can't serialize parents".to_string(),
        extra: format!("{}", err),
    })?;
    let data = serde_json::to_string(&doc.data).map_err(|err| RepositoryError::DBError {
        msg: "Can't serialize data".to_string(),
        extra: format!("{}", err),
    })?;
    Ok(DocumentRow {
        id: doc.id.to_owned(),
        name: doc.name.to_owned(),
        parents,
        author: doc.author.to_owned(),
        timestamp: doc.timestamp.naive_utc(),
        type_: doc.type_.to_owned(),
        data,
        schema_id: doc.schema_id.clone(),
    })
}

fn make_head_id(name: &str, store: &str) -> String {
    format!("{}@{}", name, store)
}

impl<'a> DocumentRepository<'a> {
    pub fn new(connection: &'a StorageConnection) -> Self {
        DocumentRepository { connection }
    }

    /// Inserts a document
    pub fn insert_document(&self, doc: &Document) -> Result<(), RepositoryError> {
        diesel::insert_into(document_dsl::document)
            .values(row_from_document(doc)?)
            .execute(&self.connection.connection)?;
        Ok(())
    }

    #[cfg(feature = "postgres")]
    pub fn update_document_head(&self, store: &str, doc: &Document) -> Result<(), RepositoryError> {
        let row = DocumentHeadRow {
            id: make_head_id(&doc.name, store),
            store: store.to_owned(),
            name: doc.name.to_owned(),
            head: doc.id.to_owned(),
        };
        diesel::insert_into(document_head_dsl::document_head)
            .values(&row)
            .on_conflict(document_head_dsl::id)
            .do_update()
            .set(&row)
            .execute(&self.connection.connection)?;
        Ok(())
    }

    /// Set document head to the provided version
    #[cfg(not(feature = "postgres"))]
    pub fn update_document_head(&self, store: &str, doc: &Document) -> Result<(), RepositoryError> {
        diesel::replace_into(document_head_dsl::document_head)
            .values(DocumentHeadRow {
                id: make_head_id(&doc.name, store),
                store: store.to_owned(),
                name: doc.name.to_owned(),
                head: doc.id.to_owned(),
            })
            .execute(&self.connection.connection)?;
        Ok(())
    }

    /// Get a specific document version
    pub fn find_one_by_id(&self, document_id: &str) -> Result<Document, RepositoryError> {
        let row = document_dsl::document
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
        let head = self.head(document_name, store)?;
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

    pub fn head(
        &self,
        document_name: &str,
        store: &str,
    ) -> Result<DocumentHeadRow, RepositoryError> {
        let result: DocumentHeadRow = document_head_dsl::document_head
            .filter(document_head_dsl::id.eq(make_head_id(document_name, store)))
            .first(&self.connection.connection)?;
        Ok(result)
    }

    /// Gets ancestor details for the full document history.
    pub fn ancestor_details(
        &self,
        document_name: &str,
    ) -> Result<Vec<AncestorDetail>, RepositoryError> {
        let rows: Vec<(String, String, NaiveDateTime)> = document_dsl::document
            .filter(document_dsl::name.eq(document_name))
            .select((
                document_dsl::id,
                document_dsl::parents,
                document_dsl::timestamp,
            ))
            .load(&self.connection.connection)?;
        let mut ancestors = Vec::<AncestorDetail>::new();
        for row in rows {
            let parents: Vec<String> =
                serde_json::from_str(&row.1).map_err(|err| RepositoryError::DBError {
                    msg: "Invalid parents data".to_string(),
                    extra: format!("{}", err),
                })?;
            ancestors.push(AncestorDetail {
                id: row.0,
                parents,
                timestamp: row.2,
            })
        }
        Ok(ancestors)
    }
}
