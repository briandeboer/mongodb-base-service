use bson::{oid::ObjectId, Bson};
use serde::{ser::SerializeMap, Deserialize, Serialize, Serializer};

/// An ID as defined by the GraphQL specification
///
/// Represented as a string, but can be converted _to_ from an integer as well.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub enum ID {
    ObjectId(ObjectId),
    String(String),
    I64(i64),
}

impl Serialize for ID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ID::ObjectId(o) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("$oid", &o.to_string())?;
                map.end()
            }
            ID::String(s) => serializer.serialize_str(s),
            ID::I64(i) => serializer.serialize_i64(i.clone()),
        }
    }
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

    pub fn with_oid(value: ObjectId) -> Self {
        ID::ObjectId(value)
    }

    pub fn with_string_to_oid<S: Into<String>>(value: S) -> Self {
        let id = ObjectId::with_string(&value.into()).unwrap();
        ID::ObjectId(id)
    }

    // #[cfg(feature = "graphql")]
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

// #[cfg(feature = "graphql")]
impl From<juniper::ID> for ID {
    fn from(id: juniper::ID) -> ID {
        ID::String(id.to_string())
    }
}

// #[cfg(feature = "graphql")]
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
        println!("I am converting to an object {:?}", id);
        match id {
            ID::ObjectId(o) => o,
            ID::String(s) => ObjectId::with_string(&s).unwrap(),
            ID::I64(i) => ObjectId::with_string(&i.to_string()).unwrap(),
        }
    }
}

use juniper::{
    parser::{ParseError, ScalarToken, Token},
    InputValue, ParseScalarResult, Value,
};

graphql_scalar!(ID as "ID" where Scalar = <S>{
    resolve(&self) -> Value {
        match self {
            ID::ObjectId(ref o) => Value::scalar(format!("$oid:{}", o.to_hex())),
            ID::String(ref s) =>  Value::scalar(s.clone()),
            ID::I64(ref i) =>  Value::scalar(i.clone() as i32),
        }
    }

    from_input_value(v: &InputValue) -> Option<ID> {
        match *v {
            InputValue::Scalar(ref s) => {
                match s.as_string() {
                    Some(s) => {
                        if s.starts_with("$oid:") {
                            match ObjectId::with_string(&s[5..]) {
                                Ok(oid) => Some(ID::ObjectId(oid)),
                                Err(_) => Some(ID::String(s)),
                            }
                        } else {
                            Some(ID::String(s))
                        }
                    },
                    None => s.as_int().map(|i| ID::I64(i as i64))
                }
            }
            _ => None
        }
    }

    from_str<'a>(value: ScalarToken<'a>) -> ParseScalarResult<'a, S> {
        match value {
            ScalarToken::String(value) | ScalarToken::Int(value) => {
                Ok(S::from(value.to_owned()))
            }
            _ => Err(ParseError::UnexpectedToken(Token::Scalar(value))),
        }
    }
});
