use bson::{doc, Bson, Document};

use chrono::Utc;
use log::warn;
use mongodb::options::FindOptions;
use mongodb::Collection;
use mongodb_cursor_pagination::{CursorDirections, FindResult, PaginatedCursor};
use serde::{Deserialize, Serialize};
use voca_rs::case::snake_case;

use crate::error::ServiceError;
use crate::model::ID;

#[derive(Serialize, Deserialize)]
pub struct DeleteResponse {
    id: ID,
    success: bool,
}

use crate::model::Node;

const DEFAULT_LIMIT: i64 = 25;

fn now() -> i64 {
    Utc::now().timestamp()
}

pub trait BaseService<'a> {
    fn new(collection: &Collection, default_sort: Option<Document>) -> Self;
    fn id_parameter(&self) -> &'static str {
        "node.id"
    }
    fn data_source(&self) -> &Collection;
    fn default_sort(&self) -> Document {
        doc! { "_id": 1 }
    }
    fn default_filter(&self) -> Option<&Document> {
        None
    }
    fn default_limit(&self) -> i64 {
        DEFAULT_LIMIT
    }

    fn find<T>(
        &self,
        filter: Option<Document>,
        sort: Option<Document>,
        limit: Option<i32>,
        after: Option<String>,
        before: Option<String>,
        skip: Option<i32>,
    ) -> FindResult<T>
    where
        T: serde::Deserialize<'a>,
    {
        let coll = self.data_source();
        // build the options object
        let find_options = FindOptions::builder()
            .limit(if let Some(l) = limit {
                l as i64
            } else {
                self.default_limit()
            })
            .skip(if let Some(s) = skip { s as i64 } else { 0 })
            // TODO: make this not something arbitrary for testing purposes
            .sort(if let Some(s) = sort {
                s
            } else {
                self.default_sort()
            })
            .build();
        let is_previous_query = before.is_some() && after.is_none();
        let query_cursor = if is_previous_query {
            PaginatedCursor::new(Some(find_options), before, Some(CursorDirections::Previous))
        } else {
            PaginatedCursor::new(Some(find_options), after, None)
        };
        let find_results: FindResult<T> = if let Some(f) = filter {
            query_cursor.find(&coll, Some(&f)).unwrap()
        } else {
            query_cursor.find(&coll, self.default_filter()).unwrap()
        };
        find_results
    }

    fn search<T>(
        &self,
        search_term: String,
        fields: Vec<String>,
        sort: Option<Document>,
        limit: Option<i32>,
        after: Option<String>,
        before: Option<String>,
        skip: Option<i32>,
    ) -> FindResult<T>
    where
        T: serde::Deserialize<'a>,
    {
        let coll = self.data_source();
        // build the options object
        let find_options = FindOptions::builder()
            .limit(if let Some(l) = limit {
                l as i64
            } else {
                self.default_limit()
            })
            .skip(if let Some(s) = skip { s as i64 } else { 0 })
            // TODO: make this not something arbitrary for testing purposes
            .sort(if let Some(s) = sort {
                s
            } else {
                self.default_sort()
            })
            .build();
        let is_previous_query = before.is_some() && after.is_none();
        let query_cursor = if is_previous_query {
            PaginatedCursor::new(Some(find_options), before, Some(CursorDirections::Previous))
        } else {
            PaginatedCursor::new(Some(find_options), after, None)
        };
        let mut filter = doc! { "$or": [] };
        let or_array = filter.get_array_mut("$or").unwrap();
        for field in fields.iter().map(|f| snake_case(&f)) {
            or_array.push(Bson::Document(
                doc! { field: Bson::RegExp(search_term.clone(), "i".to_string()) },
            ));
        }
        let find_results: FindResult<T> = query_cursor.find(&coll, Some(&filter)).unwrap();
        find_results
    }

    fn find_one_by_id<T>(&self, id: ID) -> Option<T>
    where
        T: serde::Deserialize<'a>,
    {
        self.find_one_by_string_value(self.id_parameter(), &id.to_string())
    }

    fn find_one_by_string_value<T>(&self, field: &str, value: &str) -> Option<T>
    where
        T: serde::Deserialize<'a>,
    {
        let coll = self.data_source();
        let query = Some(doc! { field => value });
        let find_result = coll.find_one(query, None).unwrap();
        match find_result {
            Some(item_doc) => Some(bson::from_bson(bson::Bson::Document(item_doc)).unwrap()),
            None => None,
        }
    }

    fn find_one_by_i64<T>(&self, field: &str, value: i64) -> Option<T>
    where
        T: serde::Deserialize<'a>,
    {
        let coll = self.data_source();
        let query = Some(doc! { field => value });
        let find_result = coll.find_one(query, None).unwrap();
        match find_result {
            Some(item_doc) => Some(bson::from_bson(bson::Bson::Document(item_doc)).unwrap()),
            None => None,
        }
    }

    fn insert_embedded<T, U>(
        &self,
        id: ID,
        field_path: &str,
        new_item: T,
    ) -> Result<U, ServiceError>
    where
        T: serde::Serialize,
        U: serde::Deserialize<'a>,
    {
        // get the item
        let coll = self.data_source();
        let query = doc! { self.id_parameter(): &id.to_string() };
        let find_result = coll.find_one(Some(query.clone()), None).unwrap();

        match find_result {
            None => Err(ServiceError::NotFound("Unable to find item".into())),
            Some(_item) => {
                // insert it
                let serialized_member = bson::to_bson(&new_item)?;
                if let bson::Bson::Document(mut document) = serialized_member {
                    let mut node_details = Document::new();
                    node_details.insert("id", uuid::Uuid::new_v4().to_hyphenated().to_string());
                    node_details.insert("date_created", now());
                    node_details.insert("date_modified", now());
                    node_details.insert("created_by_id", "unknown");
                    node_details.insert("updated_by_id", "unknown");
                    document.insert("node", node_details);
                    let update_doc = doc! { "$push": { field_path: document } };
                    let _result = coll.update_one(query, update_doc, None);
                    let item_doc =
                        coll.find_one(Some(doc! { self.id_parameter() => &id.to_string() }), None)?;
                    match item_doc {
                        Some(i) => {
                            let item: U = bson::from_bson(bson::Bson::Document(i))?;
                            Ok(item)
                        }
                        None => Err(ServiceError::NotFound("Unable to find document".into())),
                    }
                } else {
                    warn!("Error converting the BSON object into a MongoDB document");
                    Err(ServiceError::ParseError(
                        "Error converting the BSON object into a MongoDB document".into(),
                    ))
                }
            }
        }
    }

    fn insert_one<T, U>(&self, new_item: T) -> Result<U, ServiceError>
    where
        T: serde::Serialize,
        U: serde::Deserialize<'a> + Node,
    {
        let coll = self.data_source();
        let serialized_member = bson::to_bson(&new_item)?;

        if let bson::Bson::Document(mut document) = serialized_member {
            let mut node_details = Document::new();
            node_details.insert("id", uuid::Uuid::new_v4().to_hyphenated().to_string());
            node_details.insert("date_created", now());
            node_details.insert("date_modified", now());
            node_details.insert("created_by_id", "unknown");
            node_details.insert("updated_by_id", "unknown");
            document.insert("node", node_details);
            let result = coll.insert_one(document, None)?; // Insert into a MongoDB collection
            let id = result.inserted_id;
            let item_doc = coll
                .find_one(Some(doc! { "_id" => id }), None)?
                .expect("Document not found");

            let item: U = bson::from_bson(bson::Bson::Document(item_doc))?;
            Ok(item)
        } else {
            warn!("Error converting the BSON object into a MongoDB document");
            Err(ServiceError::ParseError(
                "Error converting the BSON object into a MongoDB document".into(),
            ))
        }
    }

    fn delete_one_by_id(&self, id: ID) -> Result<DeleteResponse, ServiceError> {
        let coll = self.data_source();
        let filter = doc! { self.id_parameter(): id.to_string() };
        let result = coll.delete_one(filter, None);
        match result {
            Ok(r) => Ok(DeleteResponse {
                id,
                success: r.deleted_count == 1,
            }),
            Err(e) => Err(e.into()),
        }
    }

    fn delete_one_by_query(&self, filter: Document) -> Result<bool, ServiceError> {
        let coll = self.data_source();
        let result = coll.delete_one(filter, None);
        match result {
            Ok(r) => Ok(r.deleted_count == 1),
            Err(e) => Err(e.into()),
        }
    }

    fn delete_embedded(
        &self,
        id: ID,
        field_path: &str,
        embedded_id: ID,
    ) -> Result<DeleteResponse, ServiceError> {
        let coll = self.data_source();
        let query = doc! { self.id_parameter(): &id.to_string() };
        let update_doc =
            doc! { "$pull": { field_path: { self.id_parameter(): &embedded_id.to_string()} } };
        let _result = coll.update_one(query, update_doc, None)?;
        Ok(DeleteResponse {
            id: embedded_id,
            success: true,
        })
    }

    fn update_embedded<T, U>(
        &self,
        id: ID,
        field_path: &str,
        embedded_id: ID,
        update_item: T,
    ) -> Result<U, ServiceError>
    where
        T: serde::Serialize,
        U: serde::Deserialize<'a>,
    {
        let coll = self.data_source();
        let search_embedded = doc! {
            self.id_parameter(): &id.to_string(),
            format!("{}.{}", field_path, self.id_parameter()): &embedded_id.to_string(),
        };
        let serialized_member = bson::to_bson(&update_item)?;
        if let bson::Bson::Document(document) = serialized_member {
            let array_path = format!("{}.$", field_path);
            let mut update_doc = Document::new();
            for key in document.keys() {
                let value = document.get(key);
                if let Some(v) = value {
                    update_doc.insert(format!("{}.{}", array_path, key), v.clone());
                }
            }
            update_doc.insert(format!("{}.node.date_modified", array_path), now());
            update_doc.insert(format!("{}.node.updated_by_id", array_path), "unknown");
            let update = doc! { "$set": update_doc };
            let search = doc! { self.id_parameter(): &id.to_string() };
            match coll.update_one(search_embedded, update, None) {
                Ok(_res) => match coll.find_one(Some(search), None) {
                    Ok(res) => match res {
                        Some(doc) => {
                            let item: U = bson::from_bson(bson::Bson::Document(doc))?;
                            Ok(item)
                        }
                        None => Err(ServiceError::NotFound("Unable to find item".to_owned())),
                    },
                    Err(t) => {
                        warn!("Search failed");
                        Err(ServiceError::from(t))
                    }
                },
                Err(e) => Err(ServiceError::from(e)),
            }
        } else {
            Err("Unable to update document".into())
        }
    }

    fn update_one<T, U>(&self, id: ID, update_item: T) -> Result<U, ServiceError>
    where
        T: serde::Serialize,
        U: serde::Deserialize<'a> + Node,
    {
        let coll = self.data_source();
        let search = doc! { self.id_parameter(): id.to_string() };
        let serialized_member = bson::to_bson(&update_item)?;
        if let bson::Bson::Document(mut document) = serialized_member {
            document.insert("node.date_modified", now());
            document.insert("node.updated_by_id", "unknown");
            match coll.update_one(search.clone(), doc! {"$set": document}, None) {
                Ok(_res) => match coll.find_one(Some(search), None) {
                    Ok(res) => match res {
                        Some(doc) => {
                            let item: U = bson::from_bson(bson::Bson::Document(doc))?;
                            Ok(item)
                        }
                        None => Err(ServiceError::NotFound("Unable to find item".to_owned())),
                    },
                    Err(t) => {
                        warn!("Search failed");
                        Err(ServiceError::from(t))
                    }
                },
                Err(e) => Err(ServiceError::from(e)),
            }
        } else {
            Err("Invalid update document".into())
        }
    }

    fn update_one_with_doc<U>(&self, id: ID, update_doc: Document) -> Result<U, ServiceError>
    where
        U: serde::Deserialize<'a>,
    {
        let coll = self.data_source();
        let search = doc! { self.id_parameter(): id.to_string() };
        match coll.update_one(search.clone(), update_doc, None) {
            Ok(_res) => match coll.find_one(Some(search), None) {
                Ok(res) => match res {
                    Some(doc) => {
                        let item: U = bson::from_bson(bson::Bson::Document(doc))?;
                        Ok(item)
                    }
                    None => Err(ServiceError::NotFound("Unable to find item".to_owned())),
                },
                Err(t) => {
                    warn!("Search failed");
                    Err(ServiceError::from(t))
                }
            },
            Err(e) => Err(ServiceError::from(e)),
        }
    }
}
