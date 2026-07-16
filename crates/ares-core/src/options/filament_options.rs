mod gcode_source;
mod print_source;
mod region_source;
mod retract_overrides;
mod wire;

use std::fmt;

use serde::{Deserialize, Deserializer, de::Visitor};

pub use gcode_source::FilamentGCodeSourceOptions;
use gcode_source::FilamentGCodeSourceOptionsBuilder;
use print_source::FilamentPrintSourceOptionsBuilder;
pub use print_source::{FilamentPrintSourceOptions, RawOverhangFanThreshold};
pub use region_source::FilamentRegionSourceOptions;
use region_source::FilamentRegionSourceOptionsBuilder;
pub use retract_overrides::FilamentRetractOverrideOptions;
use retract_overrides::FilamentRetractOverrideOptionsBuilder;

use super::{OrcaFloat, OrcaFloats, VariantStride};

#[derive(Clone, Debug, PartialEq)]
pub struct FilamentOptions {
    pub gcode: FilamentGCodeSourceOptions,
    pub print: FilamentPrintSourceOptions,
    pub region: FilamentRegionSourceOptions,
    pub retract_overrides: FilamentRetractOverrideOptions,
    pub pellet_flow_coefficient: OrcaFloats,
}

impl FilamentOptions {
    pub(crate) fn append(&mut self, child: Self) {
        let Self {
            gcode,
            print,
            region,
            retract_overrides,
            pellet_flow_coefficient,
        } = child;
        self.gcode.append(gcode);
        self.print.append(print);
        self.region.append(region);
        self.retract_overrides.append(retract_overrides);
        self.pellet_flow_coefficient
            .0
            .extend(pellet_flow_coefficient.0);
    }
}

impl Default for FilamentOptions {
    fn default() -> Self {
        FilamentOptionsBuilder::default().resolve()
    }
}

impl<'de> Deserialize<'de> for FilamentOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(FilamentVisitor)
    }
}

struct FilamentVisitor;

impl<'de> Visitor<'de> for FilamentVisitor {
    type Value = FilamentOptions;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Orca filament options")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut builder = FilamentOptionsBuilder::default();
        while let Some(key) = map.next_key::<String>()? {
            if !builder.deserialize_known_field(&key, &mut map)? {
                return Err(serde::de::Error::custom(format!(
                    "unknown Orca filament option {key}"
                )));
            }
        }
        Ok(builder.resolve())
    }
}

#[derive(Clone, Default, PartialEq)]
pub(crate) struct FilamentOptionsBuilder {
    gcode: FilamentGCodeSourceOptionsBuilder,
    print: FilamentPrintSourceOptionsBuilder,
    region: FilamentRegionSourceOptionsBuilder,
    retract_overrides: FilamentRetractOverrideOptionsBuilder,
    pellet_flow_coefficient: Option<OrcaFloats>,
}

impl FilamentOptionsBuilder {
    pub(crate) fn resolve_profile_root(self) -> Result<FilamentOptions, crate::SliceError> {
        let mut target = self.resolve();
        let defaults = FilamentOptions::default();
        let count = target.gcode.filament_extruder_variant.0.len();
        target
            .gcode
            .normalize_profile_root(&defaults.gcode, count)?;
        target
            .print
            .normalize_profile_root(&defaults.print, count)?;
        target
            .region
            .normalize_profile_root(&defaults.region, count)?;
        target
            .retract_overrides
            .normalize_profile_root(&defaults.retract_overrides, count)?;
        Ok(target)
    }

    pub(crate) fn apply_profile_child(
        mut self,
        target: &mut FilamentOptions,
    ) -> Result<(), crate::SliceError> {
        let count = self.gcode.profile_variant_count();
        self.gcode.normalize_profile_child(count)?;
        self.print.normalize_profile_child(count)?;
        self.region.normalize_profile_child(count)?;
        self.retract_overrides.normalize_profile_child(count)?;

        let identity = self.gcode.take_profile_identity();
        let mapping =
            profile_variant_mapping(&target.gcode.filament_extruder_variant, identity.as_ref());
        let Self {
            gcode,
            print,
            region,
            retract_overrides,
            pellet_flow_coefficient,
        } = self;
        if let Some(value) = pellet_flow_coefficient {
            target.pellet_flow_coefficient = value;
        }
        gcode.apply_profile_child(&mut target.gcode, &mapping)?;
        print.apply_profile_child(&mut target.print, &mapping)?;
        region.apply_profile_child(&mut target.region, &mapping)?;
        retract_overrides.apply_profile_child(&mut target.retract_overrides, &mapping)?;
        Ok(())
    }

    pub(crate) fn is_known_field(key: &str) -> bool {
        FilamentGCodeSourceOptionsBuilder::is_known_field(key)
            || FilamentPrintSourceOptionsBuilder::is_known_field(key)
            || FilamentRegionSourceOptionsBuilder::is_known_field(key)
            || FilamentRetractOverrideOptionsBuilder::is_known_field(key)
            || key == "pellet_flow_coefficient"
    }

    pub(crate) fn deserialize_known_field<'de, A>(
        &mut self,
        key: &str,
        map: &mut A,
    ) -> Result<bool, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        if self.gcode.deserialize_known_field(key, map)?
            || self.print.deserialize_known_field(key, map)?
            || self.region.deserialize_known_field(key, map)?
            || self.retract_overrides.deserialize_known_field(key, map)?
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
        if FilamentGCodeSourceOptionsBuilder::is_known_field(key) {
            self.gcode.deserialize_known_value(key, deserializer)
        } else if FilamentPrintSourceOptionsBuilder::is_known_field(key) {
            self.print.deserialize_known_value(key, deserializer)
        } else if FilamentRegionSourceOptionsBuilder::is_known_field(key) {
            self.region.deserialize_known_value(key, deserializer)
        } else if FilamentRetractOverrideOptionsBuilder::is_known_field(key) {
            self.retract_overrides
                .deserialize_known_value(key, deserializer)
        } else if key == "pellet_flow_coefficient" {
            if self.pellet_flow_coefficient.is_some() {
                return Err(serde::de::Error::custom(
                    "duplicate Orca option pellet_flow_coefficient",
                ));
            }
            self.pellet_flow_coefficient = Some(
                serde::Deserialize::deserialize(deserializer).map_err(|error| {
                    serde::de::Error::custom(format_args!(
                        "invalid Orca option pellet_flow_coefficient: {error}"
                    ))
                })?,
            );
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn deserialize_direct_field<'de, A>(&mut self, key: &str, map: &mut A) -> Result<bool, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        if key != "pellet_flow_coefficient" {
            return Ok(false);
        }
        if self.pellet_flow_coefficient.is_some() {
            return Err(serde::de::Error::custom(
                "duplicate Orca option pellet_flow_coefficient",
            ));
        }
        self.pellet_flow_coefficient = Some(map.next_value::<OrcaFloats>().map_err(|error| {
            serde::de::Error::custom(format_args!(
                "invalid Orca option pellet_flow_coefficient: {error}"
            ))
        })?);
        Ok(true)
    }

    pub(crate) fn resolve(self) -> FilamentOptions {
        FilamentOptions {
            gcode: self.gcode.resolve(),
            print: self.print.resolve(),
            region: self.region.resolve(),
            retract_overrides: self.retract_overrides.resolve(),
            pellet_flow_coefficient: self
                .pellet_flow_coefficient
                .unwrap_or_else(|| OrcaFloats(vec![OrcaFloat(0.4157)])),
        }
    }
}

fn profile_variant_mapping(
    source: &VariantStride,
    child: Option<&VariantStride>,
) -> Vec<Option<usize>> {
    if source.0.is_empty() {
        return vec![Some(0)];
    }
    let Some(child) = child.filter(|identity| !identity.0.is_empty()) else {
        let mut mapping = vec![None; source.0.len()];
        mapping[0] = Some(0);
        return mapping;
    };
    source
        .0
        .iter()
        .map(|variant| child.0.iter().position(|candidate| candidate == variant))
        .collect()
}
