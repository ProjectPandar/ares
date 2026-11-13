use crate::options::OrcaFloatOrPercentsNullable;

mod enums;
mod wire;

use std::fmt;

use serde::{Deserialize, Deserializer, de::Visitor};

pub use enums::{
    ProcessAdaptiveInfillGradient, ProcessAdaptiveInfillMode, ProcessCenterOfSurfacePattern,
    ProcessCounterboreHoleBridging, ProcessEnsureVerticalShellThickness, ProcessFuzzySkinMode,
    ProcessFuzzySkinType, ProcessIroningType, ProcessNoiseType, ProcessSeamScarfType,
    ProcessSurfaceFillOrder, ProcessTopSurfaceExpansionDirection, ProcessWallDirection,
    ProcessWallSequence,
};

use super::super::{
    FloatOrPercent, Nullable, OrcaBool, OrcaBoolsNullable, OrcaFloat, OrcaFloatsNullable, OrcaInt,
    OrcaInts, OrcaString, OrcaStrings, Percent, option_group::declare_option_group,
    region_fields::region_option_fields,
};
use super::object_source::ProcessInfillPattern;

macro_rules! declare_process_region_source_options {
    ($($field:ident => $key:literal: $ty:ty = $default:expr),* $(,)?) => {
        declare_option_group! {
            pub struct ProcessRegionSourceOptions, ProcessRegionSourceOptionsBuilder {
                $($field => $key: $ty = $default),*,
                hole_to_polyhole_max_edges => "hole_to_polyhole_max_edges": OrcaInt = OrcaInt(50),
                top_surface_fill_order => "top_surface_fill_order": ProcessSurfaceFillOrder = ProcessSurfaceFillOrder::Default,
                bottom_surface_fill_order => "bottom_surface_fill_order": ProcessSurfaceFillOrder = ProcessSurfaceFillOrder::Default,
                top_layer_direction => "top_layer_direction": OrcaFloat = OrcaFloat(-1.0),
                bottom_layer_direction => "bottom_layer_direction": OrcaFloat = OrcaFloat(-1.0),
                center_of_surface_pattern => "center_of_surface_pattern": ProcessCenterOfSurfacePattern = ProcessCenterOfSurfacePattern::EachSurface,
                adaptive_infill_mode => "adaptive_infill_mode": ProcessAdaptiveInfillMode = ProcessAdaptiveInfillMode::Disabled,
                adaptive_infill_interior_density => "adaptive_infill_interior_density": Percent = Percent(5.0),
                adaptive_infill_gradient => "adaptive_infill_gradient": ProcessAdaptiveInfillGradient = ProcessAdaptiveInfillGradient::Linear,
                separated_infills => "separated_infills": OrcaBool = OrcaBool(false),
                small_support_perimeter_speed => "small_support_perimeter_speed": OrcaFloatOrPercentsNullable = OrcaFloatOrPercentsNullable(vec![Nullable::Value(FloatOrPercent::Percent(Percent(50.0)))]),
                small_support_perimeter_threshold => "small_support_perimeter_threshold": OrcaFloatsNullable = OrcaFloatsNullable(vec![Nullable::Value(OrcaFloat(0.0))]),
            }
        }
        impl ProcessRegionSourceOptions {
            pub const DECLARATION_ORDER: [&'static str; 171] = [
                $($key),*,
                "hole_to_polyhole_max_edges",
                "top_surface_fill_order",
                "bottom_surface_fill_order",
                "top_layer_direction",
                "bottom_layer_direction",
                "center_of_surface_pattern",
                "adaptive_infill_mode",
                "adaptive_infill_interior_density",
                "adaptive_infill_gradient",
                "separated_infills",
                "small_support_perimeter_speed",
                "small_support_perimeter_threshold",
            ];
        }
    };
}

region_option_fields!(declare_process_region_source_options);

impl ProcessRegionSourceOptionsBuilder {
    pub(crate) fn set_derived_is_infill_first(&mut self, value: OrcaBool) {
        self.is_infill_first = Some(value);
    }
}

impl Default for ProcessRegionSourceOptions {
    fn default() -> Self {
        ProcessRegionSourceOptionsBuilder::default().resolve()
    }
}

impl<'de> Deserialize<'de> for ProcessRegionSourceOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(RegionSourceVisitor)
    }
}

struct RegionSourceVisitor;

impl<'de> Visitor<'de> for RegionSourceVisitor {
    type Value = ProcessRegionSourceOptions;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Orca process region-source options")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut builder = ProcessRegionSourceOptionsBuilder::default();
        while let Some(key) = map.next_key::<String>()? {
            if !builder.deserialize_known_field(&key, &mut map)? {
                return Err(serde::de::Error::custom(format!(
                    "unknown Orca process region option {key}"
                )));
            }
        }
        Ok(builder.resolve())
    }
}

fn string(value: &str) -> OrcaString {
    OrcaString(value.to_owned())
}

fn strings(values: &[&str]) -> OrcaStrings {
    OrcaStrings(values.iter().map(|value| (*value).to_owned()).collect())
}
