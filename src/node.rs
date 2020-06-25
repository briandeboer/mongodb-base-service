use crate::id::ID;
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
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

    pub fn created_by_id(&self) -> &Option<ID> {
        &self.created_by_id
    }

    pub fn updated_by_id(&self) -> &Option<ID> {
        &self.updated_by_id
    }
}

pub trait Node {
    fn node(&self) -> &NodeDetails;
}
