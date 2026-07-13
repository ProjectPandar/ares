use super::super::super::{
    AuthorizationType, DefaultBedType, ExtruderVariantLists,
    NozzleVolumeTypes, NullableFloats, OrcaBool, OrcaFloat, OrcaFloats, OrcaInt, OrcaString,
    OrcaStrings, Point2d, Point2dGroups, Point2dList, PrintHostType, PrinterModel, PrinterNotes,
    PrinterRemainingOptions, PrinterTechnology, ThumbnailDefinitions,
};
use crate::GCodeThumbnailFormat;

pub(super) fn assert_concrete_types(value: &PrinterRemainingOptions) {
    let _: &OrcaFloat = &value.adaptive_bed_mesh_margin;
    let _: &OrcaBool = &value.bbl_use_printhost;
    let _: &OrcaString = &value.bed_custom_model;
    let _: &OrcaString = &value.bed_custom_texture;
    let _: &Point2dList = &value.bed_exclude_area;
    let _: &Point2d = &value.bed_mesh_max;
    let _: &Point2d = &value.bed_mesh_min;
    let _: &Point2d = &value.bed_mesh_probe_distance;
    let _: &Point2d = &value.best_object_pos;
    let _: &DefaultBedType = &value.default_bed_type;
    let _: &NozzleVolumeTypes = &value.default_nozzle_volume_type;
    let _: &OrcaString = &value.default_print_profile;
    let _: &OrcaFloat = &value.extruder_clearance_height_to_lid;
    let _: &OrcaFloat = &value.extruder_clearance_height_to_rod;
    let _: &OrcaFloat = &value.extruder_clearance_radius;
    let _: &Point2dGroups = &value.extruder_printable_area;
    let _: &NullableFloats = &value.extruder_printable_height;
    let _: &ExtruderVariantLists = &value.extruder_variant_list;
    let _: &OrcaString = &value.flashforge_serial_number;
    let _: &OrcaFloats = &value.grab_length;
    let _: &Point2dList = &value.head_wrap_detect_zone;
    let _: &PrintHostType = &value.host_type;
    let _: &OrcaFloat = &value.nozzle_height;
    let _: &NullableFloats = &value.nozzle_volume;
    let _: &OrcaStrings = &value.parallel_printheads_bed_exclude_areas;
    let _: &OrcaInt = &value.parallel_printheads_count;
    let _: &OrcaBool = &value.pellet_modded_printer;
    let _: &OrcaFloat = &value.preferred_orientation;
    let _: &Point2dList = &value.printable_area;
    let _: &OrcaFloat = &value.printable_height;
    let _: &OrcaString = &value.printer_agent;
    let _: &PrinterModel = &value.printer_model;
    let _: &PrinterNotes = &value.printer_notes;
    let _: &PrinterTechnology = &value.printer_technology;
    let _: &OrcaString = &value.printer_variant;
    let _: &AuthorizationType = &value.printhost_authorization_type;
    let _: &OrcaBool = &value.printhost_ssl_ignore_revoke;
    let _: &OrcaBool = &value.support_parallel_printheads;
    let _: &ThumbnailDefinitions = &value.thumbnails;
    let _: &GCodeThumbnailFormat = &value.thumbnails_format;
    let _: &OrcaStrings = &value.upward_compatible_machine;
    let _: &OrcaFloat = &value.z_offset;
}
