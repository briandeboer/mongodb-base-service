use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::ops::Deref;

/// An ID as defined by the GraphQL specification
///
/// Represented as a string, but can be converted _to_ from an integer as well.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ID(String);

impl From<String> for ID {
    fn from(s: String) -> ID {
        ID(s)
    }
}

impl ID {
    /// Construct a new ID from anything implementing `Into<String>`
    pub fn new<S: Into<String>>(value: S) -> Self {
        ID(value.into())
    }
}

impl Deref for ID {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

#[cfg(feature = "graphql")]
impl From<juniper::ID> for ID {
    fn from(id: juniper::ID) -> ID {
        ID(id.to_string())
    }
}

#[cfg(feature = "graphql")]
impl From<ID> for juniper::ID {
    fn from(id: ID) -> juniper::ID {
        juniper::ID::new(id.to_string())
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct NodeDetails {
    pub id: ID,
    date_created: i64,
    date_modified: i64,
    created_by_id: ID,
    updated_by_id: ID,
}

impl NodeDetails {
    pub fn id(&self) -> ID {
        self.id.to_owned()
    }

    pub fn date_created(&self) -> DateTime<Utc> {
        Utc.timestamp(self.date_created, 0)
    }

    pub fn date_modified(&self) -> DateTime<Utc> {
        Utc.timestamp(self.date_modified, 0)
    }

    pub fn created_by_id(&self) -> ID {
        self.created_by_id.to_owned()
    }

    pub fn updated_by_id(&self) -> ID {
        self.updated_by_id.to_owned()
    }
}

pub trait Node {
    fn node(&self) -> &NodeDetails;
}

/// Returns the first item in an array of Nodes
/// Useful for when you have two fields one that is embedded in a list and one that is not
/// ie. my_pets and favorite_pet - you probably don't want to embed it twice
pub fn get_selected_or_first<T>(selected: &Option<ID>, list: &Option<Vec<T>>) -> Option<T>
where
    T: Clone + Node,
{
    match selected {
        Some(id) => {
            // find it in the array
            match list {
                Some(list_items) => {
                    let item: Option<&T> = list_items.iter().find(|x| x.node().id == *id);
                    match item {
                        Some(i) => Some(i.clone()),
                        None => None,
                    }
                }
                None => None,
            }
        }
        None => {
            // take the first one if nothing was picked
            if let Some(list_items) = &list {
                if !list_items.is_empty() {
                    Some(list_items[0].clone())
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}
