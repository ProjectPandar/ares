use serde::{Deserialize, Serialize};

use super::{FloatOrPercent, Nullable, OrcaBool, OrcaFloat, OrcaInt, Percent};

macro_rules! typed_vector {
    ($name:ident, $item:ty) => {
        #[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
        #[serde(transparent)]
        pub struct $name(pub Vec<$item>);
    };
}

typed_vector!(OrcaBools, OrcaBool);
typed_vector!(OrcaInts, OrcaInt);
typed_vector!(OrcaFloats, OrcaFloat);
typed_vector!(OrcaPercents, Percent);
typed_vector!(OrcaFloatOrPercents, FloatOrPercent);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PrinterTechnology {
    #[serde(rename = "FFF")]
    Fff,
    #[serde(rename = "SLA")]
    Sla,
}

typed_vector!(PrinterTechnologies, PrinterTechnology);
typed_vector!(NullablePrinterTechnologies, Nullable<PrinterTechnology>);
