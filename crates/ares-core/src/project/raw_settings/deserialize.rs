use serde::{
    Deserializer,
    de::{IntoDeserializer, Visitor, value::MapDeserializer, value::SeqDeserializer},
};

use super::RawConfigValue;

impl<'de> IntoDeserializer<'de, serde_json::Error> for RawConfigValue {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self {
        self
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
