use std::fmt;

use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{SeqAccess, Visitor},
    ser::SerializeSeq,
};

use super::scalar::{OrcaFloat, format_number};
use super::semantic::serialize_string_vector;

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct OrcaString(pub String);

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize)]
#[serde(transparent)]
pub struct OrcaStrings(pub Vec<String>);

impl Serialize for OrcaStrings {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_string_vector(&self.0, serializer)
    }
}

macro_rules! opaque_strings {
    ($($name:ident),+ $(,)?) => {
        $(
            #[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize)]
            #[serde(transparent)]
            pub struct $name(pub Vec<String>);

            impl Serialize for $name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    serialize_string_vector(&self.0, serializer)
                }
            }
        )+
    };
}

opaque_strings!(
    AmsCounts,
    RammingParameters,
    CsvTable,
    SpaceTuple,
    VariantStride,
);

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FlatMatrix(pub Vec<f64>);

impl Serialize for FlatMatrix {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for value in &self.0 {
            if !value.is_finite() {
                return Err(serde::ser::Error::custom(
                    "Orca numeric value must be finite",
                ));
            }
            sequence.serialize_element(&format_number(*value))?;
        }
        sequence.end()
    }
}

impl<'de> Deserialize<'de> for FlatMatrix {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MatrixVisitor;

        impl<'de> Visitor<'de> for MatrixVisitor {
            type Value = FlatMatrix;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an array of finite Orca floats")
            }

            fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                collect_matrix(&mut sequence).map(FlatMatrix)
            }
        }

        deserializer.deserialize_seq(MatrixVisitor)
    }
}

fn collect_matrix<'de, A>(sequence: &mut A) -> Result<Vec<f64>, A::Error>
where
    A: SeqAccess<'de>,
{
    let mut values = Vec::with_capacity(sequence.size_hint().unwrap_or(0));
    while let Some(value) = sequence.next_element::<OrcaFloat>()? {
        values.push(value.0);
    }
    Ok(values)
}
