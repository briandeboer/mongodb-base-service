#[cfg(feature = "graphql")]
#[macro_use]
extern crate juniper;

use bson::Document;

mod base;
mod error;
mod id;
mod mongo;
mod node;

pub use crate::error::ServiceError;
use crate::mongo::MongoService;

use mongodb::Collection;
use std::collections::HashMap;

pub use base::{BaseService, DeleteResponse};
pub use id::ID;
pub use node::Node;
pub use node::NodeDetails;

#[cfg(feature = "graphql")]
pub use base::DeleteResponseGQL;

#[cfg(feature = "test")]
pub use base::mock_time;

#[derive(Clone)]
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
