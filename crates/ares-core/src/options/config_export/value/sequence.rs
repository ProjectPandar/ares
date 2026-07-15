use serde::{
    Serialize,
    ser::{SerializeSeq, SerializeTuple, SerializeTupleStruct},
};

use super::{
    ConfigValueError, SerializedConfigValue, raw_string::RawStringSerializer,
    serialize_config_value, strings::serialize_string_vector, token,
};

#[derive(Clone, Copy)]
pub(super) enum SequenceKind {
    Ordinary,
    Strings,
    PointGroups,
    Nullable,
}

pub(super) struct ConfigSequence {
    kind: SequenceKind,
    values: Vec<SerializedConfigValue>,
}

impl ConfigSequence {
    pub(super) fn new(kind: SequenceKind, length: Option<usize>) -> Self {
        Self {
            kind,
            values: Vec::with_capacity(length.unwrap_or(0)),
        }
    }

    fn push<T>(&mut self, value: &T) -> Result<(), ConfigValueError>
    where
        T: Serialize + ?Sized,
    {
        let value = match self.kind {
            SequenceKind::Strings => token(value.serialize(RawStringSerializer)?),
            _ => serialize_config_value(value)?,
        };
        self.values.push(value);
        Ok(())
    }

    fn finish(self) -> SerializedConfigValue {
        match self.kind {
            SequenceKind::Strings => SerializedConfigValue {
                token: serialize_string_vector(&self.values),
                is_nil: false,
            },
            SequenceKind::PointGroups => joined(self.values, "#", false),
            SequenceKind::Nullable => {
                let is_nil = self.values.iter().all(|value| value.is_nil);
                joined(self.values, ",", is_nil)
            }
            SequenceKind::Ordinary => joined(self.values, ",", false),
        }
    }
}

impl SerializeSeq for ConfigSequence {
    type Ok = SerializedConfigValue;
    type Error = ConfigValueError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.push(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.finish())
    }
}

impl SerializeTuple for ConfigSequence {
    type Ok = SerializedConfigValue;
    type Error = ConfigValueError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.push(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.finish())
    }
}

impl SerializeTupleStruct for ConfigSequence {
    type Ok = SerializedConfigValue;
    type Error = ConfigValueError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.push(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.finish())
    }
}

fn joined(
    values: Vec<SerializedConfigValue>,
    separator: &str,
    is_nil: bool,
) -> SerializedConfigValue {
    SerializedConfigValue {
        token: values
            .into_iter()
            .map(|value| value.token)
            .collect::<Vec<_>>()
            .join(separator),
        is_nil,
    }
}
