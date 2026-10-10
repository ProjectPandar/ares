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
