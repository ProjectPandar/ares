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
    ($name:ident, $item:ty, $marker:ident) => {
        #[derive(Clone, Debug, Default, PartialEq, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub Vec<$item>);

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
    CONFIG_OPTION_NULLABLE_VECTOR
);
numeric_vector!(
    OrcaFloatsNullable,
    Nullable<OrcaFloat>,
    CONFIG_OPTION_NULLABLE_VECTOR
);

impl OrcaFloatsNullable {
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
