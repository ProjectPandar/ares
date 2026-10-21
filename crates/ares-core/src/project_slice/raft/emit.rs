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

use super::fill_params::flow_spacing;
use super::fills::{self, RaftFillSpec, raft_layer_fill};
use super::grid::{RaftLayerKind, RaftLayerZ};
use crate::geometry::{CoordinateScale, Point, Polygon};
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
    let interface_ids = interface_ids(raft_grid);
    for (index, z) in raft_grid.iter().enumerate() {
        let (polygons, spec) = match z.kind {
            RaftLayerKind::Base if index == 0 => {
                // Base flange (`SupportCommon.cpp:1496-1500`),
                // `with_sheath = true` (`:1516`).
                (
                    raft_polygons.first_layer.clone(),
                    RaftFillSpec {
                        angle: fill_params.flange_angle,
                        spacing: first_layer_flow_spacing_mm,
                        sheath_spacing: first_layer_flow_spacing_mm,
                        density: (first_layer_density_percent * 0.01) as f32,
                        // The flange uses `filler_interface` (`:1450,1496-1500`).
                        pattern: fill_params.interface_fill_pattern,
                        sheath: true,
                        // `no_sort = true` (`SupportCommon.cpp:1517`).
                        no_sort: true,
                        tree_double_wall_area: fill_params.tree_double_wall_area,
                        width_mm: first_layer_width_mm,
                        preprocess: fills::RaftFillPreprocess::Closing,
                    },
                )
            }
            RaftLayerKind::Base => (
                raft_polygons.base.clone(),
                RaftFillSpec {
                    angle: fill_params.base_angle,
                    spacing: support_flow_spacing_mm,
                    sheath_spacing: flow_spacing(support_width_mm, z.height),
                    density: fill_params.support_density as f32,
                    pattern: fill_params.base_fill_pattern,
                    sheath: fill_params.with_sheath,
                    no_sort: false,
                    tree_double_wall_area: fill_params.tree_double_wall_area,
                    width_mm: support_width_mm,
                    preprocess: fills::RaftFillPreprocess::Closing,
                },
            ),
            RaftLayerKind::Interface | RaftLayerKind::Contact => {
                let angle = fill_params.interface_angle_for_layer(interface_ids[index]);
                (
                    // The contact layer extrudes the raw contact polygons;
                    // the +0.5mm `inflate_factor_fine` expansion only builds
                    // the interface silhouette underneath
                    // (`SupportCommon.cpp:287-308` vs `:1696-1738`).
                    if z.kind == RaftLayerKind::Contact {
                        raft_polygons.contact.clone()
                    } else {
                        raft_polygons.interface.clone()
                    },
                    RaftFillSpec {
                        angle,
                        spacing: support_flow_spacing_mm,
                        sheath_spacing: flow_spacing(interface_width_mm, z.height),
                        density: fill_params.raft_interface_density as f32,
                        pattern: fill_params.interface_fill_pattern,
                        sheath: false,
                        no_sort: false,
                        tree_double_wall_area: fill_params.tree_double_wall_area,
                        width_mm: interface_width_mm,
                        // `extrude_interface` wraps the contact input in
                        // `union_safety_offset_ex` (`:1729`); plain
                        // interface layers take the `closing_ex(ε)` path
                        // (`:725-727`).
                        preprocess: if z.kind == RaftLayerKind::Contact {
                            fills::RaftFillPreprocess::SafetyOffset
                        } else {
                            fills::RaftFillPreprocess::Closing
                        },
                    },
                )
            }
        };
        plans.push(RaftLayerPlan {
            z: *z,
            polygons,
            spec,
            width_mm: if index == 0 && z.kind == RaftLayerKind::Base {
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

/// Inputs from `SupportCommon.cpp:1440-1523 flow dimensions and Slicing.cpp:232-236 Z shift`.
pub(crate) struct RaftDimensions {
    pub(crate) object_print_z_min: f64,
    pub(crate) line_width_mm: f64,
    pub(crate) interface_line_width_mm: f64,
    pub(crate) layer_height_mm: f64,
}

/// Inputs from `SupportCommon.cpp:1440-1523 raft fill reference and coordinate adapter`.
pub(crate) struct RaftCoordinateFrame {
    pub(crate) reference: Point,
    pub(crate) scale: CoordinateScale,
}

/// Convert the plans to `LayerPrintPaths` and shift the object layers.
#[expect(dead_code, reason = "wired by the pipeline integration slice")]
pub(crate) fn apply_raft_to_stream(
    plans: &[RaftLayerPlan],
    object_layers: Vec<LayerPrintPaths>,
    RaftDimensions {
        object_print_z_min,
        line_width_mm,
        interface_line_width_mm,
        layer_height_mm,
    }: RaftDimensions,
    RaftCoordinateFrame { reference, scale }: RaftCoordinateFrame,
    context: &mut crate::SlicerContext,
) -> Result<Vec<LayerPrintPaths>, crate::geometry::ClipperError> {
    let raft_count = plans.len();
    let mut stream = Vec::with_capacity(raft_count + object_layers.len());
    for (index, plan) in plans.iter().enumerate() {
        let polylines = raft_layer_fill(&plan.polygons, plan.spec, reference, scale, context)?;
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

/// Per-layer `SupportLayer::interface_id` (`SupportCommon.cpp:1369-1406`).
fn interface_ids(raft_grid: &[RaftLayerZ]) -> Vec<usize> {
    const EPSILON: f64 = 1e-4;
    let mut ids: Vec<usize> = Vec::with_capacity(raft_grid.len());
    let mut layer_id_interface = 0;
    for layer in raft_grid {
        let id = match layer.kind {
            // Contacts-only layer: one past the layer it rests on
            // (`:1391-1402`), without advancing the counter.
            RaftLayerKind::Contact => raft_grid
                .iter()
                .zip(&ids)
                .find(|(other, _)| (other.print_z - layer.bottom_z).abs() < EPSILON)
                .map_or(layer_id_interface, |(_, &id)| id + 1),
            RaftLayerKind::Interface => {
                layer_id_interface += 1;
                layer_id_interface - 1
            }
            RaftLayerKind::Base => layer_id_interface,
        };
        ids.push(id);
    }
    ids
}

#[cfg(test)]
mod tests;
