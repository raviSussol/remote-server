use crate::database::repository::RepositoryError;
use crate::database::schema::{DatabaseRow, ItemRow};
use crate::server::service::graphql::schema::types::{ItemFilter, StringFilter};

use log::info;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ItemRepository {
    mock_data: Arc<Mutex<HashMap<String, DatabaseRow>>>,
}

impl ItemRepository {
    pub fn new(mock_data: Arc<Mutex<HashMap<String, DatabaseRow>>>) -> ItemRepository {
        ItemRepository { mock_data }
    }

    pub async fn insert_one(&self, item: &ItemRow) -> Result<(), RepositoryError> {
        info!("Inserting item record (item.id={})", item.id);
        self.mock_data
            .lock()
            .unwrap()
            .insert(item.id.to_string(), DatabaseRow::Item(item.clone()));
        Ok(())
    }

    pub async fn find_many(&self, filter: ItemFilter) -> Result<Vec<ItemRow>, RepositoryError> {
        let build_begins_with_filterer = |filter: Option<String>| -> Box<dyn Fn(String) -> bool> {
            Box::new(move |string: String| -> bool {
                if let Some(begins_with) = filter.clone() {
                    if !string.starts_with(&begins_with) {
                        return false;
                    }
                }
                return true;
            })
        };

        let build_ends_with_filterer = |filter: Option<String>| -> Box<dyn Fn(String) -> bool> {
            Box::new(move |string: String| -> bool {
                if let Some(ends_with) = filter.clone() {
                    if !string.ends_with(&ends_with) {
                        return false;
                    }
                }
                return true;
            })
        };

        let build_string_filterer = |filter: Option<StringFilter>| -> Box<dyn Fn(String) -> bool> {
            if let Some(filter) = filter.clone() {
                let begins_with_filter = build_begins_with_filterer(filter.begins_with);
                let ends_with_filter = build_ends_with_filterer(filter.ends_with);
                return Box::new(move |string: String| -> bool {
                    begins_with_filter(string.clone()) && ends_with_filter(string.clone())
                });
            } else {
                return Box::new(move |_string: String| -> bool { true });
            }
        };

        let build_item_filterer = |filter: ItemFilter| -> Box<dyn Fn(&ItemRow) -> bool> {
            let item_name_filterer = build_string_filterer(filter.item_name);
            Box::new(move |row: &ItemRow| -> bool { item_name_filterer(row.item_name.clone()) })
        };

        let item_filterer = build_item_filterer(filter);

        let filter_item = |row: &DatabaseRow| -> Option<ItemRow> {
            if let DatabaseRow::Item(item) = row {
                if item_filterer(item) {
                    Some(item.clone())
                } else {
                    None
                }
            } else {
                None
            }
        };

        Ok(self
            .mock_data
            .lock()
            .unwrap()
            .values()
            .filter_map(filter_item)
            .collect())
    }

    pub async fn find_one_by_id(&self, id: &str) -> Result<ItemRow, RepositoryError> {
        info!("Querying item record (item.id={})", id);
        match self.mock_data.lock().unwrap().get(&id.to_string()) {
            Some(DatabaseRow::Item(item)) => Ok(item.clone()),
            _ => Err(RepositoryError {
                msg: String::from(format!("Failed to find item record (item.id={})", id)),
            }),
        }
    }

    pub async fn find_many_by_id(&self, ids: &[String]) -> Result<Vec<ItemRow>, RepositoryError> {
        info!("Querying multiple item records (item.id={:?})", ids);
        let mut items = vec![];
        ids.iter().for_each(|id| {
            if let Some(DatabaseRow::Item(item)) = self.mock_data.lock().unwrap().get(id) {
                items.push(item.clone());
            }
        });
        Ok(items)
    }
}
