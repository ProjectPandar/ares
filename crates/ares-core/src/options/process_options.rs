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
            if !builder.deserialize_known_field(&key, &mut map)? {
                return Err(serde::de::Error::custom(format!(
                    "unknown Orca process option {key}"
                )));
            }
        }
        Ok(builder.resolve())
    }
}

#[derive(Default)]
pub(crate) struct ProcessOptionsBuilder {
    gcode: ProcessGCodeSourceOptionsBuilder,
    object: ProcessObjectSourceOptionsBuilder,
    print: ProcessPrintSourceOptionsBuilder,
    region: ProcessRegionSourceOptionsBuilder,
    ironing_expansion: Option<OrcaFloat>,
}

impl ProcessOptionsBuilder {
    pub(crate) fn is_known_field(key: &str) -> bool {
        ProcessObjectSourceOptionsBuilder::is_known_field(key)
            || ProcessRegionSourceOptionsBuilder::is_known_field(key)
            || ProcessGCodeSourceOptionsBuilder::is_known_field(key)
            || ProcessPrintSourceOptionsBuilder::is_known_field(key)
            || key == "ironing_expansion"
    }

    pub(crate) fn deserialize_known_field<'de, A>(
        &mut self,
        key: &str,
        map: &mut A,
    ) -> Result<bool, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        if self.object.deserialize_known_field(key, map)?
            || self.region.deserialize_known_field(key, map)?
            || self.gcode.deserialize_known_field(key, map)?
            || self.print.deserialize_known_field(key, map)?
        {
            Ok(true)
        } else {
            self.deserialize_direct_field(key, map)
        }
    }

    pub(crate) fn deserialize_known_value<'de, D>(
        &mut self,
        key: &str,
        deserializer: D,
    ) -> Result<bool, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if ProcessObjectSourceOptionsBuilder::is_known_field(key) {
            self.object.deserialize_known_value(key, deserializer)
        } else if ProcessRegionSourceOptionsBuilder::is_known_field(key) {
            self.region.deserialize_known_value(key, deserializer)
        } else if ProcessGCodeSourceOptionsBuilder::is_known_field(key) {
            self.gcode.deserialize_known_value(key, deserializer)
        } else if ProcessPrintSourceOptionsBuilder::is_known_field(key) {
            self.print.deserialize_known_value(key, deserializer)
        } else if key == "ironing_expansion" {
            if self.ironing_expansion.is_some() {
                return Err(serde::de::Error::custom(
                    "duplicate Orca option ironing_expansion",
                ));
            }
            self.ironing_expansion = Some(serde::Deserialize::deserialize(deserializer).map_err(
                |error| {
                    serde::de::Error::custom(format_args!(
                        "invalid Orca option ironing_expansion: {error}"
                    ))
                },
            )?);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub(crate) fn apply_derived_support_style(&mut self) {
        self.object
            .set_derived_support_style(ProcessSupportStyle::TreeHybrid);
    }

    pub(crate) fn apply_derived_is_infill_first(&mut self) {
        self.region
            .set_derived_is_infill_first(super::OrcaBool(true));
    }

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

    pub(crate) fn resolve(self) -> ProcessOptions {
        ProcessOptions {
            gcode: self.gcode.resolve(),
            object: self.object.resolve(),
            print: self.print.resolve(),
            region: self.region.resolve(),
            ironing_expansion: self.ironing_expansion.unwrap_or(OrcaFloat(0.0)),
        }
    }
}
