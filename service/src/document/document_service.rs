use chrono::Utc;
use domain::document::{AncestorDetail, Document};
use repository::{DocumentRepository, RepositoryError, StorageConnection};

use super::{
    common_ancestor::{common_ancestors, AncestorDB, CommonAncestorError, InMemoryAncestorDB},
    merge::{three_way_merge, two_way_merge, TakeLatestConflictSolver},
    raw_document::RawDocument,
};

#[derive(Debug)]
pub enum DocumentInsertError {
    /// Document version needs to be merged first. Contains an automerged document which can be
    /// reviewed and/or inserted.
    MergeRequired(RawDocument),
    InvalidDocumentHistory,
    /// Unable to finalise a document (assign an id)
    FinalisationError(String),
    DatabaseError(RepositoryError),
}

pub struct DocumentService<'a> {
    connection: &'a StorageConnection,
}

impl From<RepositoryError> for DocumentInsertError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound => DocumentInsertError::InvalidDocumentHistory,
            _ => DocumentInsertError::DatabaseError(err),
        }
    }
}

fn two_way_document_merge(our: RawDocument, their: Document) -> RawDocument {
    let our_data = our.data;
    let their_data = their.data;

    let solver = TakeLatestConflictSolver::new(our.timestamp.clone(), their.timestamp.clone());
    let merged = two_way_merge(&our_data, &their_data, &solver);

    let mut new_parents = our.parents;
    new_parents.push(their.id);

    RawDocument {
        parents: new_parents,
        timestamp: Utc::now(),
        data: merged,

        // keep exiting
        name: our.name,
        type_: our.type_,
        author: our.author,
        schema_id: our.schema_id,
    }
}

fn three_way_document_merge(our: RawDocument, their: Document, base: Document) -> RawDocument {
    let our_data = our.data;
    let their_data = their.data;
    let base_data = base.data;

    let solver = TakeLatestConflictSolver::new(our.timestamp.clone(), their.timestamp.clone());
    let merged = three_way_merge(&our_data, &their_data, &base_data, &solver);

    let mut new_parents = our.parents;
    new_parents.push(their.id);

    RawDocument {
        parents: new_parents,
        timestamp: Utc::now(),
        data: merged,

        name: our.name,
        author: our.author,
        type_: our.type_,
        schema_id: our.schema_id,
    }
}

impl<'a> DocumentService<'a> {
    pub fn new(connection: &'a StorageConnection) -> Self {
        DocumentService { connection }
    }

    pub fn get_document(&self, store: &str, name: &str) -> Result<Document, RepositoryError> {
        DocumentRepository::new(self.connection).find_one_by_name(name, store)
    }

    /// Must be called in a transaction
    pub fn insert_document(
        &self,
        store: &str,
        doc: RawDocument,
    ) -> Result<Document, DocumentInsertError> {
        let repo = DocumentRepository::new(self.connection);
        let head_option =
            repo.head(&doc.name, store)
                .map(|v| Some(v))
                .or_else(|err| match err {
                    RepositoryError::NotFound => Ok(None),
                    _ => return Err(DocumentInsertError::DatabaseError(err)),
                })?;
        // do a unchecked insert of the doc and update the head
        let insert_doc_and_head = |raw_doc: RawDocument| -> Result<Document, DocumentInsertError> {
            let doc = raw_doc
                .finalise()
                .map_err(|err| DocumentInsertError::FinalisationError(err))?;
            repo.insert_document(&doc)?;
            repo.update_document_head(store, &doc)?;
            Ok(doc)
        };
        let head = match head_option {
            Some(head) => {
                if doc.parents.contains(&head.head) {
                    return Ok(insert_doc_and_head(doc)?);
                }
                head
            }
            None => {
                if doc.parents.is_empty() {
                    return Ok(insert_doc_and_head(doc)?);
                }
                return Err(DocumentInsertError::InvalidDocumentHistory);
            }
        };

        // Leaving the happy path; propose a auto merged doc:
        // 1) if has common ancestor -> 3 way merge
        // 2) else -> 2 way merge

        // prepare some common data:
        let their_doc = repo.find_one_by_id(&head.head)?;
        let mut db = InMemoryAncestorDB::new();
        db.insert(&repo.ancestor_details(&doc.name)?);

        // use our latest parent to find the common ancestor
        let mut our_parents = Vec::<AncestorDetail>::new();
        for parent in &doc.parents {
            match db.get_details(parent) {
                Some(detail) => our_parents.push(detail),
                None => return Err(DocumentInsertError::InvalidDocumentHistory),
            }
        }
        our_parents.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        let latest_parent = our_parents.first();
        let latest_parent_id = match latest_parent {
            Some(p) => p.id.to_owned(),
            None => {
                // no parents try a two way merge
                let merged = two_way_document_merge(doc, their_doc);
                return Err(DocumentInsertError::MergeRequired(merged));
            }
        };
        let ancestor = match common_ancestors(&db, &latest_parent_id, &their_doc.id) {
            Ok(a) => Some(a),
            Err(err) => match err {
                CommonAncestorError::NoCommonAncestorFound => None,
                CommonAncestorError::InvalidAncestorData => {
                    return Err(DocumentInsertError::InvalidDocumentHistory);
                }
            },
        };

        match ancestor {
            Some(base) => {
                let base_doc = repo.find_one_by_id(&base)?;
                let merged = three_way_document_merge(doc, their_doc, base_doc);
                Err(DocumentInsertError::MergeRequired(merged))
            }
            None => {
                // no common ancestor try a two way merge
                let merged = two_way_document_merge(doc, their_doc);
                Err(DocumentInsertError::MergeRequired(merged))
            }
        }
    }
}

#[cfg(test)]
mod document_service_test {
    use assert_json_diff::assert_json_eq;
    use chrono::{DateTime, NaiveDateTime, Utc};
    use repository::{get_storage_connection_manager, test_db};
    use serde_json::json;

    use crate::document::raw_document::RawDocument;

    use super::*;

    #[actix_rt::test]
    async fn test_insert_and_auto_resolve_conflict() {
        let settings = test_db::get_test_db_settings("omsupply-database-document_service");
        test_db::setup(&settings).await;
        let connection_manager = get_storage_connection_manager(&settings);
        let connection = connection_manager.connection().unwrap();

        let service = DocumentService::new(&connection);
        let store = "test_store";
        let template = RawDocument {
            name: "test/doc".to_string(),
            parents: vec![],
            author: "me".to_string(),
            timestamp: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(5000, 0), Utc),
            type_: "test_data".to_string(),
            data: json!({}),
            schema_id: None,
        };

        let mut base_doc = template.clone();
        base_doc.data = json!({
          "value1": "base",
          "map": {},
          "conflict": "base value"
        });
        let v0 = service.insert_document(store, base_doc).unwrap();
        // assert document is there:
        let result = service.get_document(store, &template.name).unwrap();
        assert_eq!(result.id, v0.id);

        // concurrent edits form "their" and "our"

        let mut their_doc = template.clone();
        their_doc.parents = vec![v0.id.to_owned()];
        their_doc.timestamp =
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(5100, 0), Utc);
        their_doc.data = json!({
          "value1": "their change",
          "map": {
            "entry_their": 1
          },
          "conflict": "their change"
        });
        let v1 = service.insert_document(store, their_doc).unwrap();
        let result = service.get_document(store, &template.name).unwrap();
        assert_eq!(result.id, v1.id);

        let mut our_doc = template.clone();
        our_doc.parents = vec![v0.id.to_owned()];
        our_doc.timestamp = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(5200, 0), Utc);
        our_doc.data = json!({
          "value1": "base",
          "map": {
            "entry_our": 2
          },
          "conflict": "our change wins because we are more recent"
        });
        let merge_err = service.insert_document(store, our_doc).unwrap_err();
        let auto_merge = match merge_err {
            DocumentInsertError::MergeRequired(auto_merge) => auto_merge,
            err => panic!(
                "Expected DocumentInsertError::MergeRequired but got: {:?}",
                err
            ),
        };
        // try to insert the auto merge
        service.insert_document(store, auto_merge).unwrap();
        let result = service.get_document(store, &template.name).unwrap();
        assert_json_eq!(
            result.data,
            json!({
              "value1": "their change",
              "map": {
                "entry_their": 1,
                "entry_our": 2
              },
              "conflict": "our change wins because we are more recent"
            })
        );
        assert_eq!(result.parents, vec![v0.id.to_owned(), v1.id.to_owned()]);

        // add new doc with a merge as parent
        let mut next_doc = template.clone();
        next_doc.parents = vec![result.id.to_owned()];
        next_doc.timestamp = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(5500, 0), Utc);
        next_doc.data = json!({
          "value1": "next change",
          "map": {
            "entry_their": 1,
            "entry_our": 2
          },
          "conflict": "our change wins because we are more recent"
        });
        let v4 = service.insert_document(store, next_doc).unwrap();
        let result = service.get_document(store, &template.name).unwrap();
        assert_eq!(result.id, v4.id);
        assert_json_eq!(
            result.data,
            json!({
              "value1": "next change",
              "map": {
                "entry_their": 1,
                "entry_our": 2
              },
              "conflict": "our change wins because we are more recent"
            })
        );
    }
}
