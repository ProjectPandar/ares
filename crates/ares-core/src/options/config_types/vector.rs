use serde::{Deserialize, Serialize, Serializer};

use super::{FloatOrPercent, Nullable, OrcaBool, OrcaFloat, OrcaInt, Percent};

macro_rules! typed_vector {
    ($name:ident, $item:ty) => {
        #[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
        #[serde(transparent)]
        pub struct $name(pub Vec<$item>);
    };
}

// Config.hpp:837/1015/1344/1878: numeric/bool vectors own is_nil,
// unlike string, point and enum vectors. Newtype markers preserve JSON arrays.
macro_rules! numeric_vector {
    ($name:ident, $item:ty, $marker:ident $(, $deserialize:literal)?) => {
        #[derive(Clone, Debug, Default, PartialEq, Deserialize)]
        #[serde(transparent)]
        pub struct $name($(#[serde(deserialize_with = $deserialize)])? pub Vec<$item>);

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_newtype_struct(super::semantic::$marker, &self.0)
            }
        }
    };
}

numeric_vector!(OrcaBools, OrcaBool, CONFIG_OPTION_NUMERIC_VECTOR);
numeric_vector!(
    OrcaBoolsNullable,
    Nullable<OrcaBool>,
    CONFIG_OPTION_NULLABLE_VECTOR,
    "super::nullable_vector::bools"
);
numeric_vector!(
    OrcaFloatsNullable,
    Nullable<OrcaFloat>,
    CONFIG_OPTION_NULLABLE_VECTOR,
    "super::nullable_vector::floats"
);

impl OrcaFloatsNullable {
    // Config.hpp:985-995; libslic3r.h:320-323. This is the option-equality
    // guard before an overlay, not a relaxation of wire/output comparisons.
    pub(crate) fn source_equals(&self, other: &Self) -> bool {
        self.0.len() == other.0.len()
            && self.0.iter().zip(&other.0).all(|(a, b)| match (a, b) {
                (Nullable::Nil, Nullable::Nil) => true,
                (Nullable::Value(a), Nullable::Value(b)) => (a.0 - b.0).abs() < 1e-4,
                (Nullable::Nil, Nullable::Value(_)) | (Nullable::Value(_), Nullable::Nil) => false,
            })
    }

    // Config.hpp:676-682 and ConfigOptionFloatsNullable's NaN sentinel.
    pub(crate) fn get_at(&self, index: usize) -> f64 {
        match self.0.get(index).unwrap_or(&self.0[0]) {
            Nullable::Value(value) => value.0,
            Nullable::Nil => f64::NAN,
        }
    }
}

impl OrcaBoolsNullable {
    // Config.hpp:1992: the nil byte is nonzero, not an inferred false.
    pub(crate) fn get_at(&self, index: usize) -> bool {
        !matches!(
            self.0.get(index).unwrap_or(&self.0[0]),
            Nullable::Value(OrcaBool(false))
        )
    }

    // PrintConfig.hpp:887-890 includes nil in any_enabled.
    pub(crate) fn any_enabled(&self) -> bool {
        self.0
            .iter()
            .any(|value| !matches!(value, Nullable::Value(OrcaBool(false))))
    }
}
numeric_vector!(
    OrcaFloatOrPercentsNullable,
    Nullable<FloatOrPercent>,
    CONFIG_OPTION_NULLABLE_VECTOR,
    "super::nullable_vector::floats_or_percents"
);

impl OrcaFloatOrPercentsNullable {
    // Config.hpp:62–63 compares both the value and percentage tag.
    pub(crate) fn source_equals(&self, other: &Self) -> bool {
        self.0.len() == other.0.len()
            && self.0.iter().zip(&other.0).all(|(a, b)| match (a, b) {
                (Nullable::Nil, Nullable::Nil) => true,
                (
                    Nullable::Value(FloatOrPercent::Float(a)),
                    Nullable::Value(FloatOrPercent::Float(b)),
                ) => (a - b).abs() < 1e-4,
                (
                    Nullable::Value(FloatOrPercent::Percent(a)),
                    Nullable::Value(FloatOrPercent::Percent(b)),
                ) => (a.0 - b.0).abs() < 1e-4,
                (Nullable::Nil, Nullable::Value(_))
                | (Nullable::Value(_), Nullable::Nil)
                | (
                    Nullable::Value(FloatOrPercent::Float(_)),
                    Nullable::Value(FloatOrPercent::Percent(_)),
                )
                | (
                    Nullable::Value(FloatOrPercent::Percent(_)),
                    Nullable::Value(FloatOrPercent::Float(_)),
                ) => false,
            })
    }

    pub(crate) fn get_at(&self, index: usize) -> FloatOrPercent {
        match self.0.get(index).unwrap_or(&self.0[0]) {
            Nullable::Value(value) => *value,
            Nullable::Nil => FloatOrPercent::Float(f64::NAN),
        }
    }
}

numeric_vector!(OrcaInts, OrcaInt, CONFIG_OPTION_INTS);
numeric_vector!(OrcaFloats, OrcaFloat, CONFIG_OPTION_NUMERIC_VECTOR);
numeric_vector!(OrcaPercents, Percent, CONFIG_OPTION_NUMERIC_VECTOR);
numeric_vector!(
    OrcaFloatOrPercents,
    FloatOrPercent,
    CONFIG_OPTION_NUMERIC_VECTOR
);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PrinterTechnology {
    #[serde(rename = "FFF")]
    Fff,
    #[serde(rename = "SLA")]
    Sla,
}

typed_vector!(PrinterTechnologies, PrinterTechnology);
typed_vector!(NullablePrinterTechnologies, Nullable<PrinterTechnology>);
