mod enums;
mod structured;
mod wire;

use std::fmt;

use serde::{Deserialize, Deserializer, de::Visitor};

pub use enums::{AuthorizationType, NozzleVolumeType, NozzleVolumeTypes, PrintHostType};
pub use structured::{
    DefaultBedType, ExtruderVariantLists, NullableFloats, PrinterModel, PrinterNotes,
    ThumbnailDefinitions,
};

use crate::{GCodeThumbnailFormat, ThumbnailParseError};

use super::super::{
    config_types::{
        OrcaBool, OrcaFloat, OrcaFloats, OrcaInt, OrcaString, OrcaStrings, Point2d, Point2dGroups,
        Point2dList, PrinterTechnology,
    },
    option_group::declare_option_group,
};

declare_option_group! {
    pub struct PrinterRemainingOptions, PrinterRemainingOptionsBuilder {
        printable_area => "printable_area": Point2dList = points(&[(0.0, 0.0), (200.0, 0.0), (200.0, 200.0), (0.0, 200.0)]),
        extruder_printable_area => "extruder_printable_area": Point2dGroups = Point2dGroups(Vec::new()),
        support_parallel_printheads => "support_parallel_printheads": OrcaBool = OrcaBool(false),
        parallel_printheads_count => "parallel_printheads_count": OrcaInt = OrcaInt(1),
        parallel_printheads_bed_exclude_areas => "parallel_printheads_bed_exclude_areas": OrcaStrings = OrcaStrings(Vec::new()),
        bed_exclude_area => "bed_exclude_area": Point2dList = points(&[(0.0, 0.0)]),
        head_wrap_detect_zone => "head_wrap_detect_zone": Point2dList = Point2dList(Vec::new()),
        bed_custom_texture => "bed_custom_texture": OrcaString = string(""),
        bed_custom_model => "bed_custom_model": OrcaString = string(""),
        extruder_clearance_height_to_rod => "extruder_clearance_height_to_rod": OrcaFloat = OrcaFloat(40.0),
        extruder_clearance_height_to_lid => "extruder_clearance_height_to_lid": OrcaFloat = OrcaFloat(120.0),
        extruder_clearance_radius => "extruder_clearance_radius": OrcaFloat = OrcaFloat(40.0),
        nozzle_height => "nozzle_height": OrcaFloat = OrcaFloat(2.5),
        printable_height => "printable_height": OrcaFloat = OrcaFloat(100.0),
        extruder_printable_height => "extruder_printable_height": NullableFloats = nullable_floats(&[0.0]),
        best_object_pos => "best_object_pos": Point2d = Point2d::new(0.5, 0.5),
        printer_model => "printer_model": PrinterModel = PrinterModel(String::new()),
        z_offset => "z_offset": OrcaFloat = OrcaFloat(0.0),
        nozzle_volume => "nozzle_volume": NullableFloats = nullable_floats(&[0.0]),
        thumbnails => "thumbnails": ThumbnailDefinitions = ThumbnailDefinitions(string("48x48/PNG, 300x300/PNG")),
        grab_length => "grab_length": OrcaFloats = floats(&[0.0]),
        printer_notes => "printer_notes": PrinterNotes = PrinterNotes(String::new()),
        preferred_orientation => "preferred_orientation": OrcaFloat = OrcaFloat(0.0),
        bed_mesh_min => "bed_mesh_min": Point2d = Point2d::new(-99999.0, -99999.0),
        bed_mesh_max => "bed_mesh_max": Point2d = Point2d::new(99999.0, 99999.0),
        bed_mesh_probe_distance => "bed_mesh_probe_distance": Point2d = Point2d::new(50.0, 50.0),
        adaptive_bed_mesh_margin => "adaptive_bed_mesh_margin": OrcaFloat = OrcaFloat(0.0),
        printer_technology => "printer_technology": PrinterTechnology = PrinterTechnology::Fff,
        bbl_use_printhost => "bbl_use_printhost": OrcaBool = OrcaBool(false),
        printer_agent => "printer_agent": OrcaString = string(""),
        flashforge_serial_number => "flashforge_serial_number": OrcaString = string(""),
        printhost_ssl_ignore_revoke => "printhost_ssl_ignore_revoke": OrcaBool = OrcaBool(false),
        printhost_authorization_type => "printhost_authorization_type": AuthorizationType = AuthorizationType::Key,
        default_bed_type => "default_bed_type": DefaultBedType = DefaultBedType(String::new()),
        upward_compatible_machine => "upward_compatible_machine": OrcaStrings = OrcaStrings(Vec::new()),
        default_print_profile => "default_print_profile": OrcaString = string(""),
        pellet_modded_printer => "pellet_modded_printer": OrcaBool = OrcaBool(false),
        host_type => "host_type": PrintHostType = PrintHostType::OctoPrint,
        printer_variant => "printer_variant": OrcaString = string(""),
        default_nozzle_volume_type => "default_nozzle_volume_type": NozzleVolumeTypes = NozzleVolumeTypes(vec![NozzleVolumeType::Standard]),
        extruder_variant_list => "extruder_variant_list": ExtruderVariantLists = ExtruderVariantLists(vec!["Direct Drive Standard".to_owned()]),
        thumbnails_format => "thumbnails_format": GCodeThumbnailFormat = GCodeThumbnailFormat::Png,
    }
}

impl PrinterRemainingOptionsBuilder {
    pub(crate) fn normalize_present_thumbnails(&mut self) -> Result<(), ThumbnailParseError> {
        let Some(thumbnails) = self.thumbnails.as_ref() else {
            return Ok(());
        };
        self.thumbnails = Some(super::super::typed_legacy::normalize_thumbnails(
            thumbnails,
            self.thumbnails_format,
        )?);
        Ok(())
    }
}

impl PrinterRemainingOptions {
    pub const PRINT_CONFIG_DECLARATION_ORDER: [&'static str; 27] = [
        "printable_area",
        "extruder_printable_area",
        "support_parallel_printheads",
        "parallel_printheads_count",
        "parallel_printheads_bed_exclude_areas",
        "bed_exclude_area",
        "head_wrap_detect_zone",
        "bed_custom_texture",
        "bed_custom_model",
        "extruder_clearance_height_to_rod",
        "extruder_clearance_height_to_lid",
        "extruder_clearance_radius",
        "nozzle_height",
        "printable_height",
        "extruder_printable_height",
        "best_object_pos",
        "printer_model",
        "z_offset",
        "nozzle_volume",
        "thumbnails",
        "grab_length",
        "printer_notes",
        "preferred_orientation",
        "bed_mesh_min",
        "bed_mesh_max",
        "bed_mesh_probe_distance",
        "adaptive_bed_mesh_margin",
    ];

    pub const RUNTIME_REGISTRATION_ORDER: [&'static str; 15] = [
        "printer_technology",
        "bbl_use_printhost",
        "printer_agent",
        "flashforge_serial_number",
        "printhost_ssl_ignore_revoke",
        "printhost_authorization_type",
        "default_bed_type",
        "upward_compatible_machine",
        "default_print_profile",
        "pellet_modded_printer",
        "host_type",
        "printer_variant",
        "default_nozzle_volume_type",
        "extruder_variant_list",
        "thumbnails_format",
    ];
}

impl Default for PrinterRemainingOptions {
    fn default() -> Self {
        PrinterRemainingOptionsBuilder::default().resolve()
    }
}

impl<'de> Deserialize<'de> for PrinterRemainingOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(RemainingVisitor)
    }
}

struct RemainingVisitor;

impl<'de> Visitor<'de> for RemainingVisitor {
    type Value = PrinterRemainingOptions;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("remaining Orca printer options")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut builder = PrinterRemainingOptionsBuilder::default();
        while let Some(key) = map.next_key::<String>()? {
            if !builder.deserialize_known_field(&key, &mut map)? {
                return Err(serde::de::Error::custom(format!(
                    "unknown remaining Orca printer option {key}"
                )));
            }
        }
        Ok(builder.resolve())
    }
}

fn string(value: &str) -> OrcaString {
    OrcaString(value.to_owned())
}

fn floats(values: &[f64]) -> OrcaFloats {
    OrcaFloats(
        values
            .iter()
            .copied()
            .map(super::super::OrcaFloat)
            .collect(),
    )
}

fn nullable_floats(values: &[f64]) -> NullableFloats {
    NullableFloats(
        values
            .iter()
            .copied()
            .map(|value| super::super::Nullable::Value(super::super::OrcaFloat(value)))
            .collect(),
    )
}

fn points(values: &[(f64, f64)]) -> Point2dList {
    Point2dList(values.iter().map(|&(x, y)| Point2d::new(x, y)).collect())
}
