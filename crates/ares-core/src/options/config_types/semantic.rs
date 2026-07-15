use serde::{Serialize, Serializer};

use super::Nullable;

pub(crate) const CONFIG_OPTION_STRINGS: &str = "ConfigOptionStrings";
pub(crate) const CONFIG_OPTION_POINTS_GROUPS: &str = "ConfigOptionPointsGroups";
pub(crate) const CONFIG_OPTION_NULLABLE_VECTOR: &str = "ConfigOptionNullableVector";
pub(crate) const CONFIG_OPTION_NIL: &str = "ConfigOptionNil";

pub(crate) fn serialize_string_vector<S>(
    values: &[String],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_newtype_struct(CONFIG_OPTION_STRINGS, values)
}

pub(crate) fn serialize_nullable_vector<S, T>(
    values: &[Nullable<T>],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize,
{
    NullableVectorRef::new(values).serialize(serializer)
}

pub(crate) fn serialize_nil<S>(serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_newtype_struct(CONFIG_OPTION_NIL, "nil")
}

pub(crate) struct NullableVectorRef<'a, T> {
    values: &'a [Nullable<T>],
}

impl<'a, T> NullableVectorRef<'a, T> {
    pub(crate) fn new(values: &'a [Nullable<T>]) -> Self {
        Self { values }
    }
}

impl<T> Serialize for NullableVectorRef<'_, T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_newtype_struct(CONFIG_OPTION_NULLABLE_VECTOR, self.values)
    }
}
