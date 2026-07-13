mod enums;
mod wire;

use std::fmt;

use serde::{Deserialize, Deserializer, de::Visitor};

pub use enums::ProjectBedType;

use super::super::{
    FlatMatrix, OrcaBool, OrcaBools, OrcaFloat, OrcaFloats, OrcaInt, OrcaInts, OrcaStrings,
    Point2d, Point2dList, option_group::declare_option_group,
};

declare_option_group! {
    pub struct ProjectPrintSourceOptions, ProjectPrintSourceOptionsBuilder {
        curr_bed_type => "curr_bed_type": ProjectBedType = ProjectBedType::CoolPlate,
        first_layer_print_sequence => "first_layer_print_sequence": OrcaInts = ints(&[0]),
        other_layers_print_sequence => "other_layers_print_sequence": OrcaInts = ints(&[0]),
        other_layers_print_sequence_nums => "other_layers_print_sequence_nums": OrcaInt = OrcaInt(0),
        extruder_colour => "extruder_colour": OrcaStrings = strings(&[""]),
        extruder_offset => "extruder_offset": Point2dList = points(&[(0.0, 0.0)]),
        max_layer_height => "max_layer_height": OrcaFloats = floats(&[0.0]),
        min_layer_height => "min_layer_height": OrcaFloats = floats(&[0.07]),
        nozzle_diameter => "nozzle_diameter": OrcaFloats = floats(&[0.4]),
        retraction_minimum_travel => "retraction_minimum_travel": OrcaFloats = floats(&[2.0]),
        retract_when_changing_layer => "retract_when_changing_layer": OrcaBools = bools(&[false]),
        wipe => "wipe": OrcaBools = bools(&[false]),
        wipe_distance => "wipe_distance": OrcaFloats = floats(&[1.0]),
        wipe_tower_x => "wipe_tower_x": OrcaFloats = floats(&[15.0]),
        wipe_tower_y => "wipe_tower_y": OrcaFloats = floats(&[220.0]),
        flush_volumes_matrix => "flush_volumes_matrix": FlatMatrix = flush_matrix(),
        flush_volumes_vector => "flush_volumes_vector": OrcaFloats = floats(&[140.0; 8]),
        flush_multiplier => "flush_multiplier": OrcaFloats = floats(&[0.3]),
        start_end_points => "start_end_points": Point2dList = points(&[(30.0, -3.0), (54.0, 245.0)]),
    }
}

impl ProjectPrintSourceOptions {
    pub const DECLARATION_ORDER: [&'static str; 19] = [
        "curr_bed_type",
        "first_layer_print_sequence",
        "other_layers_print_sequence",
        "other_layers_print_sequence_nums",
        "extruder_colour",
        "extruder_offset",
        "max_layer_height",
        "min_layer_height",
        "nozzle_diameter",
        "retraction_minimum_travel",
        "retract_when_changing_layer",
        "wipe",
        "wipe_distance",
        "wipe_tower_x",
        "wipe_tower_y",
        "flush_volumes_matrix",
        "flush_volumes_vector",
        "flush_multiplier",
        "start_end_points",
    ];
}

impl Default for ProjectPrintSourceOptions {
    fn default() -> Self {
        ProjectPrintSourceOptionsBuilder::default().resolve()
    }
}

impl<'de> Deserialize<'de> for ProjectPrintSourceOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(PrintSourceVisitor)
    }
}

struct PrintSourceVisitor;

impl<'de> Visitor<'de> for PrintSourceVisitor {
    type Value = ProjectPrintSourceOptions;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Orca PrintConfig project options")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut builder = ProjectPrintSourceOptionsBuilder::default();
        while let Some(key) = map.next_key::<String>()? {
            if !builder.deserialize_known_field(&key, &mut map)? {
                return Err(serde::de::Error::unknown_field(
                    &key,
                    &ProjectPrintSourceOptions::DECLARATION_ORDER,
                ));
            }
        }
        Ok(builder.resolve())
    }
}

fn bools(values: &[bool]) -> OrcaBools {
    OrcaBools(values.iter().copied().map(OrcaBool).collect())
}

fn floats(values: &[f64]) -> OrcaFloats {
    OrcaFloats(values.iter().copied().map(OrcaFloat).collect())
}

fn ints(values: &[i32]) -> OrcaInts {
    OrcaInts(values.iter().copied().map(OrcaInt).collect())
}

fn strings(values: &[&str]) -> OrcaStrings {
    OrcaStrings(values.iter().map(|value| (*value).to_owned()).collect())
}

fn points(values: &[(f64, f64)]) -> Point2dList {
    Point2dList(values.iter().map(|(x, y)| Point2d::new(*x, *y)).collect())
}

fn flush_matrix() -> FlatMatrix {
    FlatMatrix(vec![
        0.0, 280.0, 280.0, 280.0, 280.0, 0.0, 280.0, 280.0, 280.0, 280.0, 0.0, 280.0, 280.0, 280.0,
        280.0, 0.0,
    ])
}
