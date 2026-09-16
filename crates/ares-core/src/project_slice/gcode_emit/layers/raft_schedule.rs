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

use crate::project_slice::raft::RaftLayerPlan;
use crate::project_slice::raft::bridge::build_project_raft;

/// Object layer indices at or above this are raft sentinels.
pub(super) const RAFT_SENTINEL_BASE: usize = usize::MAX / 2;

pub(super) struct RaftSchedule {
    /// `(z, object_index, RAFT_SENTINEL_BASE + raft_layer_index)`
    /// sorted by z.
    pub(super) entries: Vec<(f64, usize, usize)>,
    /// Per-object raft layer plans, `[object][raft_layer]`.
    pub(super) plans: Vec<Vec<RaftLayerPlan>>,
    /// Per-object materialized raft layers, `[object][raft_layer]`.
    pub(super) layers: Vec<Vec<crate::project_slice::island_print_order::OrderedExtrusionLayer>>,
    /// Per-object object-layer z shift.
    pub(super) object_z_shift: Vec<f64>,
    /// Total raft layer count per object (0 = no raft).
    pub(super) counts: Vec<usize>,
}

impl RaftSchedule {
    pub(super) const fn active(&self) -> bool {
        !self.entries.is_empty()
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "the schedule seam forwards the traversal's per-object inputs"
)]
pub(super) fn build(
    settings: &crate::ProjectSettings,
    objects: &[crate::ObjectOptions],
    first_layer_lslices: &[Option<&[crate::geometry::ExPolygon]>],
    object_heights: &[f64],
    scale: crate::geometry::CoordinateScale,
) -> Result<Option<RaftSchedule>, crate::SliceError> {
    let object_count = objects.len();
    let mut entries = Vec::new();
    let mut plans = vec![Vec::new(); object_count];
    let mut layers = (0..object_count)
        .map(|_: usize| Vec::new())
        .collect::<Vec<_>>();
    let mut object_z_shift = vec![0.0; object_count];
    let mut counts = vec![0usize; object_count];
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
            object_heights
                .get(object_index)
                .copied()
                .unwrap_or_default(),
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
        counts[object_index] = stream.plans.len();
        plans[object_index] = stream.plans;
        object_z_shift[object_index] = stream.object_print_z_min;
        layers[object_index] = plans[object_index]
            .iter()
            .enumerate()
            .filter_map(|(raft_index, plan)| materialize_layer(plan, scale, raft_index == 0))
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
        object_z_shift,
        counts,
    }))
}

/// Split a merged entry into either a raft layer or an object layer.
pub(super) fn classify(layer_index: usize) -> Result<Option<usize>, crate::SliceError> {
    if layer_index < RAFT_SENTINEL_BASE {
        return Ok(None);
    }
    let raft_index = layer_index - RAFT_SENTINEL_BASE;
    Ok(Some(raft_index))
}

/// Materialize one raft layer plan into an `OrderedExtrusionLayer`
/// (fill entities with support roles — the motion machinery emits
/// travel/Z/TYPE/wipe exactly like object layers).
fn materialize_layer(
    plan: &RaftLayerPlan,
    scale: crate::geometry::CoordinateScale,
    flange: bool,
) -> Option<crate::project_slice::island_print_order::OrderedExtrusionLayer> {
    use crate::project_slice::fill_entities::{
        FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath,
    };
    use crate::project_slice::island_print_order::{IslandPrintEntity, OrderedExtrusionIsland};
    use crate::project_slice::raft::fills::raft_layer_fill;
    use crate::{ExtrusionRole, geometry::Point};

    let role = if plan.z.kind == crate::project_slice::raft::RaftLayerKind::Base {
        ExtrusionRole::SupportMaterial
    } else {
        ExtrusionRole::SupportMaterialInterface
    };
    let width = plan.width_mm as f32;
    let height = plan.height_mm as f32;
    let mm3_per_mm = crate::project_slice::perimeters::flow::ordinary_volume(width, height);
    let make_path = |polyline| {
        FillExtrusionEntity::Path(FillExtrusionPath {
            polyline,
            fitting: Vec::new(),
            role,
            mm3_per_mm,
            width,
            height,
        })
    };

    // Flange (raft layer 0): `with_sheath = no_sort = true`
    // (`SupportCommon.cpp:1516-1517`) — a perimeter loop of the
    // `closing_ex(polygons, ε, ε + 0.5·width)` region
    // (`draw_perimeters` with a `0.15·spacing` end clip) followed by
    // the fill on `offset(−0.4·spacing)` of the same region
    // (`fill_expolygons_with_sheath_generate_paths:705-757`).
    let (fill_polygons, entities_extra, no_sort) = if flange {
        let units = 1.0 / scale.factor();
        let inflate = ((0.5 * plan.width_mm + 0.000_001) * units).round() as i64;
        let closing = crate::geometry::offset_expolygons(
            &plan
                .polygons
                .iter()
                .map(|polygon| crate::geometry::ExPolygon::new(polygon.clone(), Vec::new()))
                .collect::<Vec<_>>(),
            inflate as f32,
            crate::geometry::JoinType::Miter,
            3.0,
        )
        .ok()?;
        let clip = (0.15 * plan.spec.spacing * units) as f64;
        let mut sheath = Vec::new();
        for expolygon in &closing {
            let mut points = expolygon.contour().points().to_vec();
            points.push(points[0]);
            let mut polyline = crate::geometry::Polyline::new(points);
            clip_polyline_end(&mut polyline, clip);
            if polyline.points().len() > 1 {
                sheath.push(make_path(polyline));
            }
        }
        let fill_region = crate::geometry::offset_expolygons(
            &closing,
            -((0.4 * plan.spec.spacing) * units) as f32,
            crate::geometry::JoinType::Miter,
            3.0,
        )
        .ok()?;
        let fill_polygons = fill_region
            .iter()
            .map(|expolygon| expolygon.contour().clone())
            .collect::<Vec<_>>();
        (fill_polygons, sheath, true)
    } else {
        (plan.polygons.clone(), Vec::new(), false)
    };

    let polylines = raft_layer_fill(
        &fill_polygons,
        crate::project_slice::raft::RaftFillSpec {
            angle: plan.spec.angle,
            spacing: plan.spec.spacing,
            density: plan.spec.density,
        },
        Point::new(0, 0),
        scale,
    )
    .ok()?;
    if polylines.is_empty() && entities_extra.is_empty() {
        return None;
    }
    let mut entities = entities_extra;
    entities.extend(polylines.into_iter().map(make_path));
    Some(
        crate::project_slice::island_print_order::OrderedExtrusionLayer {
            islands: vec![OrderedExtrusionIsland {
                entities: vec![IslandPrintEntity::FillCollection(FillExtrusionCollection {
                    entities,
                    no_sort,
                    simplify_reversed: false,
                })],
            }],
        },
    )
}

/// `Polyline::clip_end` (`ClipperZUtils`/`Polyline.cpp`): shorten the
/// polyline tail by `distance`, dropping exhausted segments.
fn clip_polyline_end(polyline: &mut crate::geometry::Polyline, distance: f64) {
    let mut points = polyline.clone().into_points();
    let mut remaining = distance;
    while points.len() > 1 {
        let last = points.len() - 1;
        let (dx, dy) = (
            points[last].x() - points[last - 1].x(),
            points[last].y() - points[last - 1].y(),
        );
        let length = ((dx * dx + dy * dy) as f64).sqrt();
        if length > remaining {
            let ratio = (length - remaining) / length;
            let x = points[last - 1].x()
                + ((points[last].x() - points[last - 1].x()) as f64 * ratio).round() as i64;
            let y = points[last - 1].y()
                + ((points[last].y() - points[last - 1].y()) as f64 * ratio).round() as i64;
            points[last] = crate::geometry::Point::new(x, y);
            *polyline = crate::geometry::Polyline::new(points);
            return;
        }
        remaining -= length;
        points.pop();
    }
    *polyline = crate::geometry::Polyline::new(points);
}
