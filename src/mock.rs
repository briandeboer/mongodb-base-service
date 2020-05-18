use bson::{doc, Document};
use mongodb::Collection;

use crate::base::BaseService;

#[derive(Clone)]
pub struct MockService {
    data_source: Collection,
    default_sort: Option<Document>,
}

impl BaseService<'_> for MongoService {
    fn new(collection: &Collection, default_sort: Option<Document>) -> Self {
        MongoService {
            data_source: collection.clone(),
            default_sort,
        }
    }
    fn data_source(&self) -> &Collection {
        &self.data_source
    }
    fn default_sort(&self) -> Document {
        match &self.default_sort {
            Some(sort) => sort.clone(),
            None => doc! { "_id": 1 },
        }
    }
}
