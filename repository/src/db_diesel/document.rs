use super::StorageConnection;

use crate::diesel_macros::apply_equal_filter;
use crate::schema::diesel_schema::{
    document::dsl as document_dsl, document_head::dsl as document_head_dsl,
};
use crate::schema::{DocumentHeadRow, DocumentRow};
use crate::RepositoryError;

use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::prelude::*;
use domain::document::{AncestorDetail, Document};
use domain::EqualFilter;

#[derive(Clone)]
pub struct DocumentFilter {
    pub name: Option<EqualFilter<String>>,
}

#[derive(Clone)]
pub struct DocumentHeadFilter {
    pub name: Option<EqualFilter<String>>,
}

pub struct DocumentRepository<'a> {
    connection: &'a StorageConnection,
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
    pub fn update_document_head(
        &self,
        store_id: &str,
        doc: &Document,
    ) -> Result<(), RepositoryError> {
        diesel::replace_into(document_head_dsl::document_head)
            .values(DocumentHeadRow {
                id: make_head_id(store_id, &doc.name),
                store: store_id.to_owned(),
                name: doc.name.to_owned(),
                head: doc.id.to_owned(),
            })
            .execute(&self.connection.connection)?;
        Ok(())
    }

    /// Get a specific document version
    pub fn find_one_by_id(&self, document_id: &str) -> Result<Option<Document>, RepositoryError> {
        let row: Option<DocumentRow> = document_dsl::document
            .filter(document_dsl::id.eq(document_id))
            .first(&self.connection.connection)
            .optional()?;

        Ok(match row {
            Some(row) => Some(document_from_row(row)?),
            None => None,
        })
    }

    /// Get the latest version of a document
    pub fn find_one_by_name(
        &self,
        store_id: &str,
        document_name: &str,
    ) -> Result<Option<Document>, RepositoryError> {
        let head = match self.head(store_id, document_name)? {
            Some(head) => head,
            None => return Ok(None),
        };
        self.find_one_by_id(&head.head)
    }

    pub fn query(
        &self,
        store_id: &str,
        filter: Option<DocumentFilter>,
    ) -> Result<Vec<Document>, RepositoryError> {
        let heads_filer = filter.map(|f| DocumentHeadFilter { name: f.name });
        let heads = self.query_heads(store_id, heads_filer)?;
        let document_ids: Vec<String> = heads.into_iter().map(|head| head.head).collect();
        let rows: Vec<DocumentRow> = document_dsl::document
            .filter(document_dsl::id.eq_any(&document_ids))
            .load(&self.connection.connection)?;

        let mut result = Vec::<Document>::new();
        for row in rows {
            result.push(document_from_row(row)?);
        }
        Ok(result)
    }

    /// Gets all document versions
    pub fn document_history(&self, document_name: &str) -> Result<Vec<Document>, RepositoryError> {
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
        store_id: &str,
        document_name: &str,
    ) -> Result<Option<DocumentHeadRow>, RepositoryError> {
        let result: Option<DocumentHeadRow> = document_head_dsl::document_head
            .filter(document_head_dsl::id.eq(make_head_id(store_id, document_name)))
            .first(&self.connection.connection)
            .optional()?;
        Ok(result)
    }

    pub fn query_heads(
        &self,
        store_id: &str,
        filter: Option<DocumentHeadFilter>,
    ) -> Result<Vec<DocumentHeadRow>, RepositoryError> {
        let filter = filter.map(|f| DocumentHeadFilter {
            name: f.name.map(|n| EqualFilter {
                equal_to: n.equal_to.map(|value| make_head_id(store_id, &value)),
                not_equal_to: n.not_equal_to.map(|value| make_head_id(store_id, &value)),
                equal_any: n.equal_any.map(|values| {
                    values
                        .iter()
                        .map(|value| make_head_id(store_id, value))
                        .collect()
                }),
            }),
        });

        let mut query = document_head_dsl::document_head.into_boxed();
        if let Some(f) = filter {
            apply_equal_filter!(query, f.name, document_head_dsl::name);
        }
        let result = query.load(&self.connection.connection)?;
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

fn make_head_id(store_id: &str, name: &str) -> String {
    format!("{}@{}", name, store_id)
}
