//! Extrusion role descriptions for the full-gcode-comment family.

use crate::PrintPathRole;

pub(super) fn extrude_description(
    gcode_comments: bool,
    role: PrintPathRole,
) -> Option<&'static str> {
    if !gcode_comments {
        return None;
    }
    Some(match role {
        PrintPathRole::Skirt => "skirt",
        PrintPathRole::Brim => "brim",
        PrintPathRole::ExternalPerimeter
        | PrintPathRole::OverhangPerimeter
        | PrintPathRole::InternalPerimeter => "perimeter",
        PrintPathRole::Ironing => "ironing",
        PrintPathRole::SupportMaterial => "support material",
        PrintPathRole::SupportMaterialInterface => "support material interface",
        _ => "infill",
    })
}
