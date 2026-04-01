use sqlx::{
    Decode, Encode, Sqlite,
    encode::IsNull,
    error::BoxDynError,
    sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef},
};
use std::{
    fmt,
    ops::{Deref, DerefMut},
    str::FromStr,
};
use ulid::{DecodeError, Ulid as ExternalUlid};

#[derive(Debug, Clone, Copy)]
pub struct Ulid(ExternalUlid);

impl Ulid {
    pub fn from_ulid(ulid: ExternalUlid) -> Self {
        Self(ulid)
    }
}

impl Default for Ulid {
    fn default() -> Self {
        Self(ExternalUlid::new())
    }
}

impl FromStr for Ulid {
    type Err = DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(ExternalUlid::from_str(s)?))
    }
}

impl Deref for Ulid {
    type Target = ExternalUlid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Ulid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl sqlx::Type<Sqlite> for Ulid {
    fn type_info() -> SqliteTypeInfo {
        <&str as sqlx::Type<Sqlite>>::type_info()
    }
}

impl Encode<'_, Sqlite> for Ulid {
    fn encode_by_ref(&self, buf: &mut Vec<SqliteArgumentValue<'_>>) -> Result<IsNull, BoxDynError> {
        buf.push(SqliteArgumentValue::Text(self.to_string().into()));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Sqlite> for Ulid {
    fn decode(value: SqliteValueRef<'_>) -> Result<Self, BoxDynError> {
        let s = <&str as Decode<Sqlite>>::decode(value)?;
        Ok(Ulid::from_ulid(ExternalUlid::from_str(s)?))
    }
}

impl serde::ser::Serialize for Ulid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

struct UlidVisitor;

impl serde::de::Visitor<'_> for UlidVisitor {
    type Value = Ulid;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string representing a ulid")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ulid::from_str(value).map_err(E::custom)
    }
}

impl<'de> serde::de::Deserialize<'de> for Ulid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(UlidVisitor)
    }
}
