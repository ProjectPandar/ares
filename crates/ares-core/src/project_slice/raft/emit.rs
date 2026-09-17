//! Raft emission wiring: builds the raft layer stream and shifts the
//! object layers, ported from the raft layer loop of
//! `generate_support_layers` + `SupportCommon.cpp:1440-1523`
//! (per-layer fill selection) and the object layer z shift
//! (`Slicing.cpp:232-236`).
//!
//! The raft layers prepend the object layer stream: object
//! `layer_id`s shift by the raft count (upstream `layer->id()`
//! incorporates the raft layers, `SupportMaterial.cpp:2130`) and
//! `print_z` shifts by `object_print_z_min`.

use super::fill_params::RaftFillInputs;
use super::fills::{RaftFillSpec, raft_layer_fill};
use super::grid::{RaftLayerKind, RaftLayerZ, raft_layer_grid};
use super::polygons::{RaftPolygonParams, raft_polygons};
use crate::geometry::{CoordinateScale, ExPolygon, Point, Polygon};
use crate::print_paths::{LayerPrintPaths, PrintPath, PrintPathRole};

#[derive(Clone, Debug)]
pub(crate) struct RaftLayerPlan {
    pub(crate) z: RaftLayerZ,
    pub(crate) polygons: Vec<Polygon>,
    pub(crate) spec: RaftFillSpec,
    /// Extrusion width in mm (support / interface role width).
    pub(crate) width_mm: f64,
    /// Layer height in mm.
    pub(crate) height_mm: f64,
}

/// Build the raft layer plans for one object
/// (`SupportCommon.cpp:1440-1523`).
#[expect(clippy::too_many_arguments, reason = "layer inputs mirror the source")]
pub(crate) fn raft_layer_plans(
    first_layer_lslices: &[ExPolygon],
    raft_grid: &[RaftLayerZ],
    raft_polygons: &super::polygons::RaftPolygons,
    fill_params: &super::fill_params::RaftFillParams,
    support_flow_spacing_mm: f64,
    first_layer_flow_spacing_mm: f64,
    first_layer_density_percent: f64,
    support_width_mm: f64,
    interface_width_mm: f64,
    first_layer_width_mm: f64,
    first_layer_height_mm: f64,
) -> Vec<RaftLayerPlan> {
    let mut plans = Vec::with_capacity(raft_grid.len());
    let mut interface_id = 0usize;
    for (index, z) in raft_grid.iter().enumerate() {
        let (polygons, spec) = match z.kind {
            RaftLayerKind::Base if index == 0 => {
                // Base flange (`SupportCommon.cpp:1496-1500`).
                (
                    raft_polygons.first_layer.clone(),
                    RaftFillSpec {
                        angle: fill_params.flange_angle,
                        spacing: first_layer_flow_spacing_mm,
                        density: (first_layer_density_percent * 0.01) as f32,
                    },
                )
            }
            RaftLayerKind::Base => (
                raft_polygons.base.clone(),
                RaftFillSpec {
                    angle: fill_params.base_angle,
                    spacing: support_flow_spacing_mm,
                    density: fill_params.support_density as f32,
                },
            ),
            RaftLayerKind::Interface | RaftLayerKind::Contact => {
                let angle = fill_params.interface_angle_for_layer(interface_id);
                interface_id += 1;
                (
                    raft_polygons.interface.clone(),
                    RaftFillSpec {
                        angle,
                        spacing: support_flow_spacing_mm,
                        density: fill_params.raft_interface_density as f32,
                    },
                )
            }
        };
        plans.push(RaftLayerPlan {
            z: *z,
            polygons,
            spec,
            width_mm: if index == 0 {
                // Base flange: `first_layer_flow` width
                // (`SupportCommon.cpp:1501-1504`).
                first_layer_width_mm
            } else if z.kind == RaftLayerKind::Base {
                // Base: `support_material_flow.width()`
                // (`:1481`).
                support_width_mm
            } else {
                // Interface/contact: `raft_interface_flow.width()`
                // (`:1512`).
                interface_width_mm
            },
            height_mm: if index == 0 {
                first_layer_height_mm
            } else {
                z.height
            },
        });
    }
    plans
}

/// Convert the plans to `LayerPrintPaths` and shift the object layers.
#[expect(dead_code, reason = "wired by the pipeline integration slice")]
pub(crate) fn apply_raft_to_stream(
    plans: &[RaftLayerPlan],
    object_layers: Vec<LayerPrintPaths>,
    object_print_z_min: f64,
    line_width_mm: f64,
    interface_line_width_mm: f64,
    layer_height_mm: f64,
    reference: Point,
    scale: CoordinateScale,
) -> Result<Vec<LayerPrintPaths>, crate::geometry::ClipperError> {
    let raft_count = plans.len();
    let mut stream = Vec::with_capacity(raft_count + object_layers.len());
    for (index, plan) in plans.iter().enumerate() {
        let polylines = raft_layer_fill(&plan.polygons, plan.spec, reference, scale)?;
        let role = if plan.z.kind == RaftLayerKind::Base {
            PrintPathRole::SupportMaterial
        } else {
            PrintPathRole::SupportMaterialInterface
        };
        let width = if plan.z.kind == RaftLayerKind::Base {
            line_width_mm
        } else {
            interface_line_width_mm
        };
        let mut paths = Vec::with_capacity(polylines.len());
        for polyline in polylines {
            let points = polyline
                .points()
                .iter()
                .map(|point| crate::Point2::new(scale.unscale(point.x()), scale.unscale(point.y())))
                .collect::<Vec<_>>();
            if let Ok(mut path) = PrintPath::new(role, points) {
                path = path.with_effective_line_width_mm(Some(width));
                path = path.with_effective_layer_height_mm(layer_height_mm);
                paths.push(path);
            }
        }
        stream.push(LayerPrintPaths::new(index, plan.z.print_z, paths));
    }
    for (index, mut layer) in object_layers.into_iter().enumerate() {
        layer = LayerPrintPaths::new(
            index + raft_count,
            layer.print_z() + object_print_z_min,
            layer.paths().to_vec(),
        );
        stream.push(layer);
    }
    Ok(stream)
}

const _: () = {
    // Silence the unused re-export until the pipeline slice lands.
    let _: Option<RaftFillInputs> = None;
};
