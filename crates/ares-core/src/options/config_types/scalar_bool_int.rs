use std::fmt;

use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error, Visitor},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrcaBool(pub bool);

impl Serialize for OrcaBool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(if self.0 { "1" } else { "0" })
    }
}

impl<'de> Deserialize<'de> for OrcaBool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BoolVisitor;

        impl<'de> Visitor<'de> for BoolVisitor {
            type Value = OrcaBool;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("Orca boolean 0 or 1")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
                Ok(OrcaBool(value))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match value.split(',').next().unwrap_or_default().trim() {
                    "0" => Ok(OrcaBool(false)),
                    "1" => Ok(OrcaBool(true)),
                    _ => Err(E::custom("expected Orca boolean 0 or 1")),
                }
            }
        }

        deserializer.deserialize_any(BoolVisitor)
    }
}

macro_rules! integer_type {
    ($name:ident, $inner:ty, $expecting:literal) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $name(pub $inner);

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.0.to_string())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct IntegerVisitor;

                impl<'de> Visitor<'de> for IntegerVisitor {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                        formatter.write_str($expecting)
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: Error,
                    {
                        value.trim().parse::<$inner>().map($name).map_err(E::custom)
                    }

                    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
                    where
                        E: Error,
                    {
                        <$inner>::try_from(value)
                            .map($name)
                            .map_err(|_| E::custom($expecting))
                    }

                    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
                    where
                        E: Error,
                    {
                        <$inner>::try_from(value)
                            .map($name)
                            .map_err(|_| E::custom($expecting))
                    }
                }

                deserializer.deserialize_any(IntegerVisitor)
            }
        }
    };
}

integer_type!(OrcaInt, i32, "a signed Orca integer");
integer_type!(OrcaUInt, u32, "an unsigned Orca integer");
