use bson::{oid::ObjectId, Bson};
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

/// An ID as defined by the GraphQL specification
///
/// Represented as a string, but can be converted _to_ from an integer as well.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ID {
    ObjectId(ObjectId),
    String(String),
    I64(i64),
}

impl From<String> for ID {
    fn from(s: String) -> ID {
        ID::String(s)
    }
}

impl From<i64> for ID {
    fn from(i: i64) -> ID {
        ID::I64(i)
    }
}

impl From<ObjectId> for ID {
    fn from(o: ObjectId) -> ID {
        ID::ObjectId(o)
    }
}

impl ID {
    pub fn from_string<S: Into<String>>(value: S) -> Self {
        ID::String(value.into())
    }

    /// Construct a new ID from anything implementing `Into<String>`
    pub fn with_string<S: Into<String>>(value: S) -> Self {
        ID::String(value.into())
    }

    pub fn with_i64<I: Into<i64>>(value: I) -> Self {
        ID::I64(value.into())
    }

    pub fn with_oid<O: Into<ObjectId>>(value: ObjectId) -> Self {
        ID::ObjectId(value.into())
    }

    pub fn with_string_to_oid<S: Into<String>>(value: S) -> Self {
        let id = ObjectId::with_string(&value.into()).unwrap();
        ID::ObjectId(id)
    }

    #[cfg(feature = "graphql")]
    pub fn with_juniper_to_oid(value: juniper::ID) -> Self {
        let id = ObjectId::with_string(&value.to_string()).unwrap();
        ID::ObjectId(id)
    }

    pub fn with_bson(value: &Bson) -> Self {
        match value.into() {
            Bson::String(s) => ID::String(s),
            Bson::ObjectId(o) => ID::ObjectId(o),
            Bson::I64(i) => ID::I64(i),
            _ => panic!("Invalid id type used {:?}", value),
        }
    }

    pub fn to_bson(&self) -> Bson {
        match self {
            ID::ObjectId(o) => Bson::ObjectId(o.clone()),
            ID::String(s) => Bson::String(s.to_string()),
            ID::I64(i) => Bson::I64(i.clone()),
        }
    }
}

#[cfg(feature = "graphql")]
impl From<juniper::ID> for ID {
    fn from(id: juniper::ID) -> ID {
        ID::String(id.to_string())
    }
}

#[cfg(feature = "graphql")]
impl From<ID> for juniper::ID {
    fn from(id: ID) -> juniper::ID {
        match id {
            ID::ObjectId(o) => juniper::ID::new(o.to_hex()),
            ID::String(s) => juniper::ID::new(s),
            ID::I64(s) => juniper::ID::new(s.to_string()),
        }
    }
}

impl From<ID> for ObjectId {
    fn from(id: ID) -> ObjectId {
        match id {
            ID::ObjectId(o) => o,
            ID::String(s) => ObjectId::with_string(&s).unwrap(),
            ID::I64(i) => ObjectId::with_string(&i.to_string()).unwrap(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NodeDetails {
    date_created: Option<i64>,
    date_modified: Option<i64>,
    created_by_id: Option<ID>,
    updated_by_id: Option<ID>,
}

fn get_timestamp(timestamp: Option<i64>) -> Option<DateTime<Utc>> {
    match timestamp {
        Some(ts) => Some(Utc.timestamp(ts, 0)),
        None => None,
    }
}

impl NodeDetails {
    pub fn date_created(&self) -> Option<DateTime<Utc>> {
        get_timestamp(self.date_created)
    }

    pub fn date_modified(&self) -> Option<DateTime<Utc>> {
        get_timestamp(self.date_modified)
    }

    pub fn created_by_id(&self) -> Option<ID> {
        self.created_by_id.to_owned()
    }

    pub fn updated_by_id(&self) -> Option<ID> {
        self.updated_by_id.to_owned()
    }
}

pub trait Node {
    fn node(&self) -> &NodeDetails;
}
