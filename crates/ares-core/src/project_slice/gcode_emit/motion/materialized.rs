use super::{EmitState, LayerGeometry, features::PathProperties, path, scarf};
use crate::project_slice::perimeters::classic::materialize::{ExtrusionPath, ExtrusionRole};

pub(super) struct Emission<'a> {
    pub(super) path: &'a ExtrusionPath,
    pub(super) end_clip: f64,
    pub(super) slope: Option<scarf::Slope>,
    pub(super) geometry: LayerGeometry<'a>,
}

pub(super) fn emit(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    emission: Emission<'_>,
) -> Result<(), crate::SliceError> {
    emit_with_speed(output, state, emission, None)
}

pub(super) fn emit_with_speed(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    emission: Emission<'_>,
    requested_speed: Option<f64>,
) -> Result<(), crate::SliceError> {
    let Emission {
        path,
        end_clip,
        slope,
        geometry,
    } = emission;
    let feature = match path.role {
        ExtrusionRole::ExternalPerimeter => "Outer wall",
        ExtrusionRole::Perimeter => "Inner wall",
        ExtrusionRole::OverhangPerimeter => "Overhang wall",
        ExtrusionRole::GapFill => "Gap infill",
        ExtrusionRole::SolidInfill => "Internal solid infill",
        ExtrusionRole::TopSolidInfill => "Top surface",
        ExtrusionRole::BottomSurface => "Bottom surface",
    };
    path::emit_with_speed(
        output,
        path.polyline.points.iter().map(|point| (point.x, point.y)),
        (
            PathProperties {
                contour: path.z_contoured.then_some(path.polyline.points.as_slice()),
                mm3_per_mm: path.mm3_per_mm,
                width: path.width,
                height: path.height,
                feature,
                is_perimeter: matches!(
                    path.role,
                    ExtrusionRole::ExternalPerimeter
                        | ExtrusionRole::Perimeter
                        | ExtrusionRole::OverhangPerimeter
                ),
                end_clip,
                fitting: &path.polyline.fitting,
                slope,
            },
            requested_speed,
        ),
        geometry,
        state,
    )
}
