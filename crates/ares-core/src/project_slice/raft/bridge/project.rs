//! `raft::bridge::build_project_raft` — project-path raft stream, split
//! from `bridge.rs` for the <400 LOC rule.

use super::super::emit::raft_layer_plans;
use super::super::fill_params::RaftFillInputs;
use super::super::grid::raft_layer_grid;
use super::super::polygons::{RaftPolygonParams, raft_polygons};
use super::{RaftStream, raft_geometry_error};
use crate::SliceError;
use crate::geometry::support_grid::SupportGridParams;
use crate::geometry::{CoordinateScale, ExPolygon};

/// Project-path raft construction using native support flow selection.
pub(in crate::project_slice) fn build_project_raft(
    settings: &crate::ProjectSettings,
    object: &crate::ObjectOptions,
    parameters: &crate::project_slice::parameters::SlicingParameters,
    first_layer_lslices: &[ExPolygon],
    scale: CoordinateScale,
) -> Result<Option<RaftStream>, SliceError> {
    let raft_layers = object.raft_layers.0.max(0) as usize;
    if raft_layers == 0 {
        return Ok(None);
    }
    let first_height = parameters.first_print_layer_height;
    let flows = crate::project_slice::perimeters::flow::resolve_support_flows(
        settings,
        object,
        first_height,
        parameters.layer_height,
    )?;
    let support_width = f64::from(flows.base.width);
    let interface_width = f64::from(flows.interface.width);
    let first_layer_width = f64::from(flows.first_layer.width);
    let support_flow_spacing = f64::from(flows.base.spacing);
    let interface_flow_spacing = f64::from(flows.interface.spacing);
    let first_layer_flow_spacing = f64::from(flows.first_layer.spacing);

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
    let raft_grid = raft_layer_grid(parameters)?;
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
    let mut plans = raft_layer_plans(
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
    let contact_pattern = super::super::fill_params::contact_fill_pattern(
        object,
        interface_flow_spacing,
        fill_params.interface_fill_pattern,
    )?;
    for plan in &mut plans {
        if plan.z.kind == super::super::grid::RaftLayerKind::Contact {
            plan.spec.pattern = contact_pattern;
            plan.spec.spacing = interface_flow_spacing;
        }
    }

    Ok(Some(RaftStream {
        plans,
        support_flow_spacing_mm: support_flow_spacing,
        first_layer_flow_spacing_mm: first_layer_flow_spacing,
    }))
}
