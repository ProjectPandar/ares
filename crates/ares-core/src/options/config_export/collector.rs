use serde::{
    Serialize, Serializer,
    ser::{Impossible, SerializeMap},
};

use super::value::{ConfigValueError, serialize_config_value};
use crate::options::ProjectSettings;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ConfigEntry {
    pub(crate) key: String,
    pub(crate) token: String,
    pub(crate) is_nil: bool,
}

pub(crate) fn collect_config_entries(
    settings: &ProjectSettings,
) -> Result<Vec<ConfigEntry>, ConfigValueError> {
    let mut entries = Vec::with_capacity(650);
    append_group(&settings.printer, &mut entries)?;
    append_group(&settings.process, &mut entries)?;
    append_group(&settings.filament, &mut entries)?;
    append_group(&settings.project, &mut entries)?;
    finish(entries)
}

fn append_group<T>(value: &T, entries: &mut Vec<ConfigEntry>) -> Result<(), ConfigValueError>
where
    T: Serialize + ?Sized,
{
    entries.extend(value.serialize(GroupSerializer)?);
    Ok(())
}

fn finish(mut entries: Vec<ConfigEntry>) -> Result<Vec<ConfigEntry>, ConfigValueError> {
    entries.sort_unstable_by(|left, right| left.key.cmp(&right.key));
    if let Some(pair) = entries.windows(2).find(|pair| pair[0].key == pair[1].key) {
        return Err(error(format_args!("duplicate config key {}", pair[0].key)));
    }
    Ok(entries)
}

#[cfg(test)]
pub(crate) fn collect_serializable_for_test<T>(
    value: &T,
) -> Result<Vec<ConfigEntry>, ConfigValueError>
where
    T: Serialize + ?Sized,
{
    finish(value.serialize(GroupSerializer)?)
}

struct GroupSerializer;

impl Serializer for GroupSerializer {
    type Ok = Vec<ConfigEntry>;
    type Error = ConfigValueError;
    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = ConfigMap;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_map(self, length: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(ConfigMap {
            entries: Vec::with_capacity(length.unwrap_or(0)),
            pending_key: None,
        })
    }

    fn serialize_bool(self, _value: bool) -> Result<Self::Ok, Self::Error> {
        Err(group_error())
    }

    fn serialize_i8(self, _value: i8) -> Result<Self::Ok, Self::Error> {
        Err(group_error())
    }

    fn serialize_i16(self, _value: i16) -> Result<Self::Ok, Self::Error> {
        Err(group_error())
    }

    fn serialize_i32(self, _value: i32) -> Result<Self::Ok, Self::Error> {
        Err(group_error())
    }

    fn serialize_i64(self, _value: i64) -> Result<Self::Ok, Self::Error> {
        Err(group_error())
    }

    fn serialize_i128(self, _value: i128) -> Result<Self::Ok, Self::Error> {
        Err(group_error())
    }

    fn serialize_u8(self, _value: u8) -> Result<Self::Ok, Self::Error> {
        Err(group_error())
    }

    fn serialize_u16(self, _value: u16) -> Result<Self::Ok, Self::Error> {
        Err(group_error())
    }

    fn serialize_u32(self, _value: u32) -> Result<Self::Ok, Self::Error> {
        Err(group_error())
    }

    fn serialize_u64(self, _value: u64) -> Result<Self::Ok, Self::Error> {
        Err(group_error())
    }

    fn serialize_u128(self, _value: u128) -> Result<Self::Ok, Self::Error> {
        Err(group_error())
    }

    fn serialize_f32(self, _value: f32) -> Result<Self::Ok, Self::Error> {
        Err(group_error())
    }

    fn serialize_f64(self, _value: f64) -> Result<Self::Ok, Self::Error> {
        Err(group_error())
    }

    fn serialize_char(self, _value: char) -> Result<Self::Ok, Self::Error> {
        Err(group_error())
    }

    fn serialize_str(self, _value: &str) -> Result<Self::Ok, Self::Error> {
        Err(group_error())
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(group_error())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(group_error())
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        Err(group_error())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(group_error())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(group_error())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(group_error())
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        Err(group_error())
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
        Err(group_error())
    }

    fn serialize_seq(self, _length: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(group_error())
    }

    fn serialize_tuple(self, _length: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(group_error())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(group_error())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(group_error())
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(group_error())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(group_error())
    }
}

struct ConfigMap {
    entries: Vec<ConfigEntry>,
    pending_key: Option<String>,
}

impl SerializeMap for ConfigMap {
    type Ok = Vec<ConfigEntry>;
    type Error = ConfigValueError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        if self.pending_key.is_some() {
            return Err(error("config map key is missing its value"));
        }
        let key = serialize_config_value(key)?;
        if key.is_nil {
            return Err(error("config map key cannot be nil"));
        }
        self.pending_key = Some(key.token);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        let key = self
            .pending_key
            .take()
            .ok_or_else(|| error("config map value is missing its key"))?;
        let value = serialize_config_value(value)?;
        self.entries.push(ConfigEntry {
            key,
            token: value.token,
            is_nil: value.is_nil,
        });
        Ok(())
    }

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        K: Serialize + ?Sized,
        V: Serialize + ?Sized,
    {
        self.serialize_key(key)?;
        self.serialize_value(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.pending_key.is_some() {
            return Err(error("config map key is missing its value"));
        }
        Ok(self.entries)
    }
}

fn group_error() -> ConfigValueError {
    error("config option group must serialize as a map")
}

fn error(message: impl std::fmt::Display) -> ConfigValueError {
    <ConfigValueError as serde::ser::Error>::custom(message)
}
