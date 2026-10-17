mod raw_string;
mod sequence;
mod strings;

use std::fmt;

use serde::{Serialize, Serializer, ser::Impossible};

use super::super::config_types::{
    format_number,
    semantic::{
        CONFIG_OPTION_INTS, CONFIG_OPTION_NIL, CONFIG_OPTION_NULLABLE_VECTOR,
        CONFIG_OPTION_NUMERIC_VECTOR, CONFIG_OPTION_POINTS_GROUPS, CONFIG_OPTION_STRINGS,
    },
};
use sequence::{ConfigSequence, SequenceKind};
pub(crate) use strings::{escape_scalar_string, escape_string_vector_element};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct SerializedConfigValue {
    pub(crate) token: String,
    pub(crate) is_nil: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ConfigValueError(String);

impl serde::ser::Error for ConfigValueError {
    fn custom<T>(message: T) -> Self
    where
        T: fmt::Display,
    {
        Self(message.to_string())
    }
}

impl fmt::Display for ConfigValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ConfigValueError {}

pub(crate) fn serialize_config_value<T>(
    value: &T,
) -> Result<SerializedConfigValue, ConfigValueError>
where
    T: Serialize + ?Sized,
{
    serialize_config_value_with_active_count(value, None)
}

pub(crate) fn serialize_config_value_with_active_count<T>(
    value: &T,
    active_count: Option<usize>,
) -> Result<SerializedConfigValue, ConfigValueError>
where
    T: Serialize + ?Sized,
{
    value.serialize(ConfigValueSerializer::ordinary(active_count))
}

fn serialize_config_value_with_sequence_kind<T>(
    value: &T,
    sequence_kind: SequenceKind,
) -> Result<SerializedConfigValue, ConfigValueError>
where
    T: Serialize + ?Sized,
{
    value.serialize(ConfigValueSerializer::with_sequence_kind(
        sequence_kind,
        None,
    ))
}

#[derive(Clone, Copy)]
struct ConfigValueSerializer {
    sequence_kind: SequenceKind,
    active_count: Option<usize>,
}

impl ConfigValueSerializer {
    const fn ordinary(active_count: Option<usize>) -> Self {
        Self {
            sequence_kind: SequenceKind::Ordinary,
            active_count,
        }
    }

    const fn with_sequence_kind(sequence_kind: SequenceKind, active_count: Option<usize>) -> Self {
        Self {
            sequence_kind,
            active_count,
        }
    }
}

impl Serializer for ConfigValueSerializer {
    type Ok = SerializedConfigValue;
    type Error = ConfigValueError;
    type SerializeSeq = ConfigSequence;
    type SerializeTuple = ConfigSequence;
    type SerializeTupleStruct = ConfigSequence;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
        Ok(token(if value { "1" } else { "0" }))
    }

    fn serialize_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        Ok(token(value.to_string()))
    }

    fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
        Ok(token(value.to_string()))
    }

    fn serialize_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
        if matches!(self.sequence_kind, SequenceKind::Ints) && value == i32::MAX {
            return Ok(SerializedConfigValue {
                token: value.to_string(),
                is_nil: true,
            });
        }
        Ok(token(value.to_string()))
    }

    fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
        Ok(token(value.to_string()))
    }

    fn serialize_i128(self, value: i128) -> Result<Self::Ok, Self::Error> {
        Ok(token(value.to_string()))
    }

    fn serialize_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
        Ok(token(value.to_string()))
    }

    fn serialize_u16(self, value: u16) -> Result<Self::Ok, Self::Error> {
        Ok(token(value.to_string()))
    }

    fn serialize_u32(self, value: u32) -> Result<Self::Ok, Self::Error> {
        Ok(token(value.to_string()))
    }

    fn serialize_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
        Ok(token(value.to_string()))
    }

    fn serialize_u128(self, value: u128) -> Result<Self::Ok, Self::Error> {
        Ok(token(value.to_string()))
    }

    fn serialize_f32(self, value: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(f64::from(value))
    }

    fn serialize_f64(self, value: f64) -> Result<Self::Ok, Self::Error> {
        if value.is_finite() {
            Ok(token(format_number(value)))
        } else {
            Err(ConfigValueError(
                "config value numbers must be finite".to_owned(),
            ))
        }
    }

    fn serialize_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(value.encode_utf8(&mut [0; 4]))
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        if matches!(self.sequence_kind, SequenceKind::Ints) && value == i32::MAX.to_string() {
            return Ok(SerializedConfigValue {
                token: value.to_owned(),
                is_nil: true,
            });
        }
        Ok(token(escape_scalar_string(value)))
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(ConfigValueError(
            "config value byte strings are unsupported".to_owned(),
        ))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(ConfigValueError(
            "config value options are unsupported".to_owned(),
        ))
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        Err(ConfigValueError(
            "config value options are unsupported".to_owned(),
        ))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(ConfigValueError(
            "config value units are unsupported".to_owned(),
        ))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(ConfigValueError(
            "config value unit structs are unsupported".to_owned(),
        ))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(token(variant))
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        match name {
            CONFIG_OPTION_NUMERIC_VECTOR => value.serialize(Self::with_sequence_kind(
                SequenceKind::Numeric,
                self.active_count,
            )),
            CONFIG_OPTION_INTS => value.serialize(Self::with_sequence_kind(
                SequenceKind::Ints,
                self.active_count,
            )),
            CONFIG_OPTION_STRINGS => value.serialize(Self::with_sequence_kind(
                SequenceKind::Strings,
                self.active_count,
            )),
            CONFIG_OPTION_POINTS_GROUPS => value.serialize(Self::with_sequence_kind(
                SequenceKind::PointGroups,
                self.active_count,
            )),
            CONFIG_OPTION_NULLABLE_VECTOR => value.serialize(Self::with_sequence_kind(
                SequenceKind::Nullable,
                self.active_count,
            )),
            CONFIG_OPTION_NIL => Ok(SerializedConfigValue {
                token: "nil".to_owned(),
                is_nil: true,
            }),
            _ => value.serialize(self),
        }
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        Err(ConfigValueError(
            "config value newtype variants are unsupported".to_owned(),
        ))
    }

    fn serialize_seq(self, length: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(ConfigSequence::new(
            self.sequence_kind,
            length,
            self.active_count,
        ))
    }

    fn serialize_tuple(self, length: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(length))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        length: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(length))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(ConfigValueError(
            "config value tuple variants are unsupported".to_owned(),
        ))
    }

    fn serialize_map(self, _length: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(ConfigValueError(
            "config value maps are unsupported".to_owned(),
        ))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(ConfigValueError(
            "config value structs are unsupported".to_owned(),
        ))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(ConfigValueError(
            "config value struct variants are unsupported".to_owned(),
        ))
    }
}

fn token(value: impl Into<String>) -> SerializedConfigValue {
    SerializedConfigValue {
        token: value.into(),
        is_nil: false,
    }
}
