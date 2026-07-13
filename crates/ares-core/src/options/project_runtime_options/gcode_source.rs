mod enums;
mod wire;

use std::fmt;

use serde::{Deserialize, Deserializer, de::Visitor};

pub use enums::ProjectFilamentMapMode;

use super::super::{
    AmsCounts, NozzleVolumeType, NozzleVolumeTypes, OrcaBool, OrcaFloat, OrcaFloats, OrcaInt,
    OrcaInts, OrcaPercents, OrcaStrings, Percent, option_group::declare_option_group,
};

declare_option_group! {
    pub struct ProjectGCodeSourceOptions, ProjectGCodeSourceOptionsBuilder {
        deretraction_speed => "deretraction_speed": OrcaFloats = floats(&[0.0]),
        filament_ids => "filament_ids": OrcaStrings = OrcaStrings::default(),
        filament_map_mode => "filament_map_mode": ProjectFilamentMapMode = ProjectFilamentMapMode::AutoForFlush,
        filament_map => "filament_map": OrcaInts = ints(&[1]),
        retract_before_wipe => "retract_before_wipe": OrcaPercents = percents(&[100.0]),
        retraction_length => "retraction_length": OrcaFloats = floats(&[0.8]),
        retract_length_toolchange => "retract_length_toolchange": OrcaFloats = floats(&[10.0]),
        z_hop => "z_hop": OrcaFloats = floats(&[0.4]),
        retract_lift_above => "retract_lift_above": OrcaFloats = floats(&[0.0]),
        retract_lift_below => "retract_lift_below": OrcaFloats = floats(&[0.0]),
        retract_restart_extra => "retract_restart_extra": OrcaFloats = floats(&[0.0]),
        retract_restart_extra_toolchange => "retract_restart_extra_toolchange": OrcaFloats = floats(&[0.0]),
        retraction_speed => "retraction_speed": OrcaFloats = floats(&[30.0]),
        nozzle_volume_type => "nozzle_volume_type": NozzleVolumeTypes = NozzleVolumeTypes(vec![NozzleVolumeType::Standard]),
        extruder_ams_count => "extruder_ams_count": AmsCounts = AmsCounts::default(),
        bbl_calib_mark_logo => "bbl_calib_mark_logo": OrcaBool = OrcaBool(true),
        has_scarf_joint_seam => "has_scarf_joint_seam": OrcaBool = OrcaBool(false),
    }
}

impl ProjectGCodeSourceOptions {
    pub const DECLARATION_ORDER: [&'static str; 17] = [
        "deretraction_speed",
        "filament_ids",
        "filament_map_mode",
        "filament_map",
        "retract_before_wipe",
        "retraction_length",
        "retract_length_toolchange",
        "z_hop",
        "retract_lift_above",
        "retract_lift_below",
        "retract_restart_extra",
        "retract_restart_extra_toolchange",
        "retraction_speed",
        "nozzle_volume_type",
        "extruder_ams_count",
        "bbl_calib_mark_logo",
        "has_scarf_joint_seam",
    ];
}

impl Default for ProjectGCodeSourceOptions {
    fn default() -> Self {
        ProjectGCodeSourceOptionsBuilder::default().resolve()
    }
}

impl<'de> Deserialize<'de> for ProjectGCodeSourceOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(GCodeSourceVisitor)
    }
}

struct GCodeSourceVisitor;

impl<'de> Visitor<'de> for GCodeSourceVisitor {
    type Value = ProjectGCodeSourceOptions;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Orca GCodeConfig project options")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut builder = ProjectGCodeSourceOptionsBuilder::default();
        while let Some(key) = map.next_key::<String>()? {
            if !builder.deserialize_known_field(&key, &mut map)? {
                return Err(serde::de::Error::unknown_field(
                    &key,
                    &ProjectGCodeSourceOptions::DECLARATION_ORDER,
                ));
            }
        }
        Ok(builder.resolve())
    }
}

fn floats(values: &[f64]) -> OrcaFloats {
    OrcaFloats(values.iter().copied().map(OrcaFloat).collect())
}

fn ints(values: &[i32]) -> OrcaInts {
    OrcaInts(values.iter().copied().map(OrcaInt).collect())
}

fn percents(values: &[f64]) -> OrcaPercents {
    OrcaPercents(values.iter().copied().map(Percent).collect())
}
