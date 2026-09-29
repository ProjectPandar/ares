//! `raft::bridge::build_project_raft` — project-path raft stream, split
//! from `bridge.rs` for the <400 LOC rule.

use super::super::emit::raft_layer_plans;
use super::super::fill_params::{RaftFillInputs, flow_spacing};
use super::super::grid::raft_layer_grid;
use super::super::polygons::{RaftPolygonParams, raft_polygons};
use super::{RaftStream, raft_geometry_error};
use crate::SliceError;
use crate::geometry::support_grid::SupportGridParams;
use crate::geometry::{CoordinateScale, ExPolygon};

/// Project-path variant: derives everything from the resolved
/// `ProjectSettings` + `ObjectOptions` (the types the project
/// pipeline carries), with the support-flow width following the
/// header's derivation (`gcode_emit/header.rs:205-221`:
/// `resolved_width(support_line_width, line_width, support_nozzle)`).
pub(crate) fn build_project_raft(
    settings: &crate::ProjectSettings,
    object: &crate::ObjectOptions,
    object_height: f64,
    first_layer_lslices: &[ExPolygon],
    scale: CoordinateScale,
) -> Result<Option<RaftStream>, SliceError> {
    let raft_layers = object.raft_layers.0.max(0) as usize;
    if raft_layers == 0 {
        return Ok(None);
    }
    let parameters =
        crate::project_slice::parameters::slicing_parameters(settings, object, object_height, &[])?;

    let nozzles = &settings.project.print.nozzle_diameter.0;
    let nozzle = nozzles.first().map_or(0.4, |value| value.0);
    let support_nozzle = {
        let index = object.support_filament.0.saturating_sub(1) as usize;
        nozzles
            .get(index)
            .or_else(|| nozzles.first())
            .map_or(nozzle, |value| value.0)
    };
    // `resolved_width(support_line_width, line_width, support_nozzle, 1.0)`
    // (`header.rs:235-244`).
    let resolve_width = |configured: crate::FloatOrPercent| -> f64 {
        explicit_width(configured, support_nozzle)
            .or_else(|| explicit_width(object.line_width, support_nozzle))
            .unwrap_or(support_nozzle)
    };
    let support_width = resolve_width(object.support_line_width);
    let interface_width = support_width; // interface width shares the support width field

    let layer_height = parameters.layer_height;
    let first_height = parameters.first_print_layer_height;
    let support_flow_spacing = flow_spacing(support_width, layer_height);
    let interface_flow_spacing = flow_spacing(interface_width, layer_height);
    // First-layer support flow follows the FIRST-LAYER width chain
    // (`initial_layer_line_width` else `line_width`, header.rs:214),
    // not `support_line_width` (oracle flange pitch 0.419 =
    // spacing(0.42)/0.9).
    let first_layer_width = explicit_width(
        settings.process.print.initial_layer_line_width,
        support_nozzle,
    )
    .or_else(|| explicit_width(object.line_width, support_nozzle))
    .unwrap_or(support_width);
    let first_layer_flow_spacing = flow_spacing(first_layer_width, first_height);

    let fill_inputs = RaftFillInputs {
        support_flow_spacing,
        interface_flow_spacing,
        base_pattern_spacing: object.support_base_pattern_spacing.0,
        interface_spacing: object.support_interface_spacing.0,
        support_angle_degrees: object.support_angle.0,
        base_raft_layers: parameters.base_raft_layers,
        interface_raft_layers: parameters.interface_raft_layers,
        // `SupportParameters.hpp:129-133`.
        with_sheath: object.tree_support_wall_count.0 > 0,
        honeycomb_base: object.support_base_pattern == crate::ProcessSupportBasePattern::Honeycomb,
        tree_double_wall_area: super::super::fill_params::tree_double_wall_area(
            object.tree_support_wall_count.0,
            1.0 / scale.factor(),
        ),
    };
    let fill_params = fill_inputs.derive();

    let units_per_mm = 1.0 / scale.factor();
    let scaled = |mm: f64| mm * units_per_mm;
    let grid = SupportGridParams::new(
        object.support_base_pattern_spacing.0 + support_flow_spacing,
        support_flow_spacing,
        units_per_mm,
    );
    let raft_grid = raft_layer_grid(&parameters)?;
    let polygons = raft_polygons(
        first_layer_lslices,
        &RaftPolygonParams {
            raft_expansion: scaled(object.raft_expansion.0),
            first_layer_expansion: scaled(object.raft_first_layer_expansion.0),
            raft_layers,
            scale: crate::project_slice::raft::polygons::RaftPolygonScale {
                units_per_mm: 1.0 / scale.factor(),
            },
            grid,
        },
    )
    .map_err(raft_geometry_error)?;
    let plans = raft_layer_plans(
        &raft_grid,
        &polygons,
        &fill_params,
        support_flow_spacing,
        first_layer_flow_spacing,
        object.raft_first_layer_density.0,
        support_width,
        interface_width,
        first_layer_width,
        first_height,
    );

    Ok(Some(RaftStream {
        plans,
        object_print_z_min: parameters.object_print_z_min,
        support_flow_spacing_mm: support_flow_spacing,
        first_layer_flow_spacing_mm: first_layer_flow_spacing,
    }))
}

/// `width()` (`header.rs:246-258`): explicit Float/Percent resolution.
fn explicit_width(value: crate::FloatOrPercent, nozzle: f64) -> Option<f64> {
    match value {
        crate::FloatOrPercent::Float(value) if value > 0.0 => Some(value),
        crate::FloatOrPercent::Percent(value) if value.0 > 0.0 => {
            Some(value.0 * f64::from(nozzle as f32) / 100.0)
        }
        _ => None,
    }
}
