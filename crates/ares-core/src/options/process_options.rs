mod gcode_source;
mod object_source;
mod print_source;
mod region_source;
mod wire;

use std::fmt;

use serde::{Deserialize, Deserializer, de::Visitor};

pub use gcode_source::ProcessGCodeSourceOptions;
pub(crate) use gcode_source::ProcessGCodeSourceOptionsBuilder;
pub(crate) use object_source::ProcessObjectSourceOptionsBuilder;
pub use object_source::{
    ProcessBrimType, ProcessExtraBridgeLayer, ProcessGapFillTarget, ProcessInfillPattern,
    ProcessInternalBridgeFilter, ProcessObjectSourceOptions, ProcessPerimeterGenerator,
    ProcessSeamPosition, ProcessSlicingMode, ProcessSupportBasePattern,
    ProcessSupportInterfacePattern, ProcessSupportStyle, ProcessSupportType,
};
pub(crate) use print_source::ProcessPrintSourceOptionsBuilder;
pub use print_source::{
    ProcessDraftShield, ProcessPrintOrder, ProcessPrintSequence, ProcessPrintSourceOptions,
    ProcessSkirtType, ProcessTimelapseType, ProcessWipeTowerWallType,
};
pub(crate) use region_source::ProcessRegionSourceOptionsBuilder;
pub use region_source::{
    ProcessCounterboreHoleBridging, ProcessEnsureVerticalShellThickness, ProcessFuzzySkinMode,
    ProcessFuzzySkinType, ProcessIroningType, ProcessNoiseType, ProcessRegionSourceOptions,
    ProcessSeamScarfType, ProcessWallDirection, ProcessWallSequence,
};

use super::OrcaFloat;

#[derive(Clone, Debug, PartialEq)]
pub struct ProcessOptions {
    pub gcode: ProcessGCodeSourceOptions,
    pub object: ProcessObjectSourceOptions,
    pub print: ProcessPrintSourceOptions,
    pub region: ProcessRegionSourceOptions,
    pub ironing_expansion: OrcaFloat,
}

impl Default for ProcessOptions {
    fn default() -> Self {
        ProcessOptionsBuilder::default().resolve()
    }
}

impl<'de> Deserialize<'de> for ProcessOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(ProcessVisitor)
    }
}

struct ProcessVisitor;

impl<'de> Visitor<'de> for ProcessVisitor {
    type Value = ProcessOptions;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Orca process options")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut builder = ProcessOptionsBuilder::default();
        while let Some(key) = map.next_key::<String>()? {
            if builder.object.deserialize_known_field(&key, &mut map)?
                || builder.region.deserialize_known_field(&key, &mut map)?
                || builder.gcode.deserialize_known_field(&key, &mut map)?
                || builder.print.deserialize_known_field(&key, &mut map)?
                || builder.deserialize_direct_field(&key, &mut map)?
            {
                continue;
            }
            return Err(serde::de::Error::custom(format!(
                "unknown Orca process option {key}"
            )));
        }
        Ok(builder.resolve())
    }
}

#[derive(Default)]
struct ProcessOptionsBuilder {
    gcode: ProcessGCodeSourceOptionsBuilder,
    object: ProcessObjectSourceOptionsBuilder,
    print: ProcessPrintSourceOptionsBuilder,
    region: ProcessRegionSourceOptionsBuilder,
    ironing_expansion: Option<OrcaFloat>,
}

impl ProcessOptionsBuilder {
    fn deserialize_direct_field<'de, A>(&mut self, key: &str, map: &mut A) -> Result<bool, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        if key != "ironing_expansion" {
            return Ok(false);
        }
        if self.ironing_expansion.is_some() {
            return Err(serde::de::Error::custom(
                "duplicate Orca option ironing_expansion",
            ));
        }
        self.ironing_expansion = Some(map.next_value::<OrcaFloat>().map_err(|error| {
            serde::de::Error::custom(format_args!(
                "invalid Orca option ironing_expansion: {error}"
            ))
        })?);
        Ok(true)
    }

    fn resolve(self) -> ProcessOptions {
        ProcessOptions {
            gcode: self.gcode.resolve(),
            object: self.object.resolve(),
            print: self.print.resolve(),
            region: self.region.resolve(),
            ironing_expansion: self.ironing_expansion.unwrap_or(OrcaFloat(0.0)),
        }
    }
}
