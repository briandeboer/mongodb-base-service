use bson::Document;

mod base;
mod error;
mod model;
mod mongo;

use crate::error::ServiceError;
use crate::mongo::MongoService;

use mongodb::Collection;
use std::collections::HashMap;

pub use base::{BaseService, DeleteResponse};
pub use model::get_selected_or_first;
pub use model::Node;
pub use model::NodeDetails;

pub struct DataSources {
    collections: HashMap<String, MongoService>,
}

impl DataSources {
    pub fn new() -> Self {
        DataSources {
            collections: HashMap::new(),
        }
    }

    pub fn create_mongo_service(
        &mut self,
        name: &str,
        collection: &Collection,
        default_sort: Option<Document>,
    ) {
        self.collections.insert(
            name.to_string(),
            MongoService::new(collection, default_sort),
        );
    }

    pub fn get_mongo_service(&self, key: &str) -> Result<&MongoService, ServiceError> {
        let service = self.collections.get(&key.to_string());
        match service {
            Some(s) => Ok(s),
            None => Err(ServiceError::ConnectionError(format!(
                "Unable to connect to collection {}",
                key
            ))),
        }
    }
}
