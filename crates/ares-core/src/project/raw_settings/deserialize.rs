use serde::{
    Deserializer,
    de::{
        IntoDeserializer, Visitor,
        value::{BorrowedStrDeserializer, MapAccessDeserializer, MapDeserializer, SeqDeserializer},
    },
};

use super::RawConfigValue;

pub(crate) enum RawValueRef<'a> {
    Borrowed(&'a RawConfigValue),
    Owned(RawConfigValue),
}

impl<'de> IntoDeserializer<'de, serde_json::Error> for &'de RawConfigValue {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self {
        self
    }
}

impl<'de> IntoDeserializer<'de, serde_json::Error> for RawValueRef<'de> {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self {
        self
    }
}

impl<'de> IntoDeserializer<'de, serde_json::Error> for RawConfigValue {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self {
        self
    }
}

impl<'de> Deserializer<'de> for &'de RawConfigValue {
    type Error = serde_json::Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self {
            RawConfigValue::Text(value) => visitor.visit_borrowed_str(value),
            RawConfigValue::Number(value) => {
                if let Some(value) = value.as_u64() {
                    visitor.visit_u64(value)
                } else if let Some(value) = value.as_i64() {
                    visitor.visit_i64(value)
                } else if let Some(value) = value.as_f64() {
                    visitor.visit_f64(value)
                } else {
                    value.clone().deserialize_any(visitor)
                }
            }
            RawConfigValue::Flag(value) => visitor.visit_bool(*value),
            RawConfigValue::List(values) => {
                SeqDeserializer::new(values.iter()).deserialize_any(visitor)
            }
            RawConfigValue::Object(values) => MapDeserializer::new(
                values
                    .iter()
                    .map(|(key, value)| (BorrowedStrDeserializer::new(key.as_str()), value)),
            )
            .deserialize_any(visitor),
            RawConfigValue::Null => visitor.visit_unit(),
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self {
            RawConfigValue::Null => visitor.visit_none(),
            value => visitor.visit_some(value),
        }
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        match self {
            RawConfigValue::Text(value) => {
                visitor.visit_enum(BorrowedStrDeserializer::new(value.as_str()))
            }
            RawConfigValue::Object(values) => MapAccessDeserializer::new(MapDeserializer::new(
                values
                    .iter()
                    .map(|(key, value)| (BorrowedStrDeserializer::new(key.as_str()), value)),
            ))
            .deserialize_enum(name, variants, visitor),
            value => value.deserialize_any(visitor),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct seq tuple tuple_struct map struct
        identifier ignored_any
    }
}

impl<'de> Deserializer<'de> for RawValueRef<'de> {
    type Error = serde_json::Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self {
            Self::Borrowed(value) => value.deserialize_any(visitor),
            Self::Owned(value) => value.deserialize_any(visitor),
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self {
            Self::Borrowed(value) => value.deserialize_option(visitor),
            Self::Owned(value) => value.deserialize_option(visitor),
        }
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        match self {
            Self::Borrowed(value) => value.deserialize_newtype_struct(name, visitor),
            Self::Owned(value) => value.deserialize_newtype_struct(name, visitor),
        }
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        match self {
            Self::Borrowed(value) => value.deserialize_enum(name, variants, visitor),
            Self::Owned(value) => value.deserialize_enum(name, variants, visitor),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct seq tuple tuple_struct map struct
        identifier ignored_any
    }
}

impl<'de> Deserializer<'de> for RawConfigValue {
    type Error = serde_json::Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self {
            Self::Text(value) => visitor.visit_string(value),
            Self::Number(value) => value.deserialize_any(visitor),
            Self::Flag(value) => visitor.visit_bool(value),
            Self::List(values) => SeqDeserializer::new(values.into_iter()).deserialize_any(visitor),
            Self::Object(values) => {
                MapDeserializer::new(values.into_iter()).deserialize_any(visitor)
            }
            Self::Null => visitor.visit_unit(),
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self {
            Self::Null => visitor.visit_none(),
            value => visitor.visit_some(value),
        }
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        match self {
            Self::Text(value) => visitor.visit_enum(value.into_deserializer()),
            Self::Object(values) => serde::de::value::MapAccessDeserializer::new(
                MapDeserializer::new(values.into_iter()),
            )
            .deserialize_enum(name, variants, visitor),
            value => value.deserialize_any(visitor),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct seq tuple tuple_struct map struct
        identifier ignored_any
    }
}
