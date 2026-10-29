//! Raft layer schedule extension for the project pipeline's merged
//! layer-chunk schedule (`layers/schedule.rs`): prepends the raft
//! layer z entries (`usize::MAX − raft_index` sentinel layer index)
//! and shifts the object layer z accumulation by
//! `object_print_z_min` (`Slicing.cpp:232-236`; upstream
//! `PrintObject::layers()` print_z carries the raft offset).
//!
//! The raft entries are consumed by the raft emission branch in
//! `layers.rs` (5b); reaching the object emission with a sentinel
//! index is a bug, so the sentinel space is asserted disjoint from
//! real object layer indices.
use crate::project_slice::perimeters::classic::materialize::Polyline3;

use crate::project_slice::raft::RaftLayerPlan;
use crate::project_slice::raft::bridge::build_project_raft;

/// Object layer indices at or above this are raft sentinels.
pub(super) const RAFT_SENTINEL_BASE: usize = usize::MAX / 2;

pub(in crate::project_slice::gcode_emit) struct RaftSchedule {
    /// `(z, object_index, RAFT_SENTINEL_BASE + raft_layer_index)`
    /// sorted by z.
    pub(super) entries: Vec<(f64, usize, usize)>,
    /// Per-object raft layer plans, `[object][raft_layer]`.
    pub(in crate::project_slice::gcode_emit) plans: Vec<Vec<RaftLayerPlan>>,
    /// Per-object materialized raft layers, `[object][raft_layer]`.
    pub(in crate::project_slice::gcode_emit) layers:
        Vec<Vec<crate::project_slice::island_print_order::OrderedExtrusionLayer>>,
}

impl RaftSchedule {
    pub(in crate::project_slice::gcode_emit) fn skirt_points(
        &self,
        object: usize,
    ) -> impl Iterator<Item = crate::geometry::Point> + '_ {
        use crate::project_slice::{
            fill_entities::FillExtrusionEntity, island_print_order::IslandPrintEntity,
        };
        self.layers[object]
            .iter()
            .flat_map(|layer| &layer.islands)
            .flat_map(|island| &island.entities)
            .flat_map(|entity| match entity {
                IslandPrintEntity::Fill(path) => std::slice::from_ref(path),
                IslandPrintEntity::FillCollection(collection) => collection.entities.as_slice(),
                IslandPrintEntity::Perimeter(_) | IslandPrintEntity::Thin(_) => {
                    unreachable!("raft materialization emits support paths")
                }
            })
            .flat_map(|entity| match entity {
                FillExtrusionEntity::Path(path) | FillExtrusionEntity::Oriented(path) => path
                    .polyline
                    .points()
                    .iter()
                    .map(|point| crate::geometry::Point::new(point.x, point.y)),
                FillExtrusionEntity::VariableWidth(_) => {
                    unreachable!("raft materialization emits constant-width paths")
                }
            })
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "the raft inputs share one job-local support connection context"
)]
pub(super) fn build(
    settings: &crate::ProjectSettings,
    objects: &[crate::ObjectOptions],
    first_layer_lslices: &[Option<&[crate::geometry::ExPolygon]>],
    slicing_parameters: &[&crate::project_slice::parameters::SlicingParameters],
    scale: crate::geometry::CoordinateScale,
    context: &mut crate::SlicerContext,
) -> Result<Option<RaftSchedule>, crate::SliceError> {
    let object_count = objects.len();
    let mut entries = Vec::new();
    let mut plans = vec![Vec::new(); object_count];
    let mut layers = (0..object_count)
        .map(|_: usize| Vec::new())
        .collect::<Vec<_>>();
    for object_index in 0..object_count {
        if objects[object_index].raft_layers.0 <= 0 {
            continue;
        }
        let lslices = first_layer_lslices
            .get(object_index)
            .copied()
            .flatten()
            .unwrap_or(&[]);
        let Some(stream) = build_project_raft(
            settings,
            &objects[object_index],
            slicing_parameters[object_index],
            lslices,
            scale,
        )?
        else {
            continue;
        };
        for (raft_index, plan) in stream.plans.iter().enumerate() {
            entries.push((
                plan.z.print_z,
                object_index,
                RAFT_SENTINEL_BASE + raft_index,
            ));
        }
        plans[object_index] = stream.plans;
        layers[object_index] = plans[object_index]
            .iter()
            .filter_map(|plan| materialize_layer(plan, scale, context))
            .collect();
    }
    if entries.is_empty() {
        return Ok(None);
    }
    entries.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    Ok(Some(RaftSchedule {
        entries,
        plans,
        layers,
    }))
}

/// Materialize one raft layer plan into an `OrderedExtrusionLayer`
/// (fill entities with support roles — the motion machinery emits
/// travel/Z/TYPE/wipe exactly like object layers).
fn materialize_layer(
    plan: &RaftLayerPlan,
    scale: crate::geometry::CoordinateScale,
    context: &mut crate::SlicerContext,
) -> Option<crate::project_slice::island_print_order::OrderedExtrusionLayer> {
    use crate::ExtrusionRole;
    use crate::project_slice::fill_entities::{
        FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath,
    };
    use crate::project_slice::island_print_order::{IslandPrintEntity, OrderedExtrusionIsland};
    use crate::project_slice::raft::fills::{RaftEntity, raft_layer_entities};

    let role = if plan.z.kind == crate::project_slice::raft::RaftLayerKind::Base {
        ExtrusionRole::SupportMaterial
    } else {
        ExtrusionRole::SupportMaterialInterface
    };
    let width = plan.width_mm as f32;
    let height = plan.height_mm as f32;
    let mm3_per_mm = crate::project_slice::perimeters::flow::ordinary_volume(width, height);
    let make_path = |polyline| FillExtrusionPath {
        polyline: Polyline3::from_xy(polyline, Vec::new()),
        z_contoured: false,
        role,
        mm3_per_mm,
        width,
        height,
    };

    // `support_fills.entities` (`SupportCommon.cpp:1476-1520`): the flange
    // regions are `no_sort` collections, tree loops are oriented paths.
    let entities = raft_layer_entities(&plan.polygons, plan.spec, scale, context).ok()?;
    if entities.is_empty() {
        return None;
    }
    let entities: Vec<_> = entities
        .into_iter()
        .map(|entity| match entity {
            RaftEntity::Path(polyline) => {
                IslandPrintEntity::Fill(FillExtrusionEntity::Path(make_path(polyline)))
            }
            RaftEntity::Oriented(polyline) => {
                IslandPrintEntity::Fill(FillExtrusionEntity::Oriented(make_path(polyline)))
            }
            RaftEntity::NoSort(polylines) => {
                IslandPrintEntity::FillCollection(FillExtrusionCollection {
                    entities: polylines
                        .into_iter()
                        .map(|polyline| FillExtrusionEntity::Path(make_path(polyline)))
                        .collect(),
                    no_sort: true,
                })
            }
        })
        .collect();
    Some(
        crate::project_slice::island_print_order::OrderedExtrusionLayer {
            tool_regions: Vec::new(),
            islands: vec![OrderedExtrusionIsland {
                infill_regions: vec![0; entities.len()],
                infill_collections: (0..entities.len()).collect(),
                perimeter_regions: Vec::new(),
                entities,
            }],
        },
    )
}

/// Raft support paths pass through the same DP simplification as object
/// support paths (`PrintObject::simplify_extrusion_path` /
/// `simplify_support_extrusion_path` over the support layers).
pub(super) fn simplify_layers(
    traversal: &crate::project_slice::perimeters::classic::traversal::types::PreparedPostClassicTraversal,
    raft_layers: &mut [Vec<crate::project_slice::island_print_order::OrderedExtrusionLayer>],
) {
    let process = &traversal.resolved.views.full.process;
    let arc_fitting = process.gcode.enable_arc_fitting.0 && !process.print.spiral_mode.0;
    let tolerance = process.print.resolution.0;
    for object_layers in raft_layers {
        crate::project_slice::path_simplification::simplify_layers(
            object_layers,
            traversal.scale,
            tolerance,
            arc_fitting,
        );
    }
}
