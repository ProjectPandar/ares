use std::fmt;

use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error, IntoDeserializer, Visitor},
};

use super::semantic::serialize_nil;

#[derive(Clone, Debug, PartialEq)]
pub enum Nullable<T> {
    Nil,
    Value(T),
}

impl<T: Serialize> Serialize for Nullable<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Nil => serialize_nil(serializer),
            Self::Value(value) => value.serialize(serializer),
        }
    }
}

impl<'de, T: serde::de::DeserializeOwned> Deserialize<'de> for Nullable<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct NullableVisitor<T>(std::marker::PhantomData<T>);

        impl<'de, T: serde::de::DeserializeOwned> Visitor<'de> for NullableVisitor<T> {
            type Value = Nullable<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("nil or a typed Orca value")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                parse_nullable(value)
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: Error,
            {
                T::deserialize(value.into_deserializer()).map(Nullable::Value)
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                T::deserialize(value.into_deserializer()).map(Nullable::Value)
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                T::deserialize(value.into_deserializer()).map(Nullable::Value)
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                T::deserialize(value.into_deserializer()).map(Nullable::Value)
            }
        }

        deserializer.deserialize_any(NullableVisitor(std::marker::PhantomData))
    }
}

fn parse_nullable<T, E>(value: &str) -> Result<Nullable<T>, E>
where
    T: serde::de::DeserializeOwned,
    E: Error,
{
    let value = value.trim();
    if value == "nil" {
        Ok(Nullable::Nil)
    } else {
        T::deserialize(value.into_deserializer()).map(Nullable::Value)
    }
}
