//! Per-loop seam placement (`SeamPlacer::place_seam_in_loop`,
//! `SeamPlacer.cpp`) split from `seam_placement.rs`.

use super::{
    alignment, mesh,
    split::{closest_projection, next_loop_point, normalized, scale_position, split_at},
};
use crate::{
    geometry::CoordinateScale,
    project_slice::{
        island_print_order::{IslandPrintEntity, OrderedExtrusionLayer},
        perimeters::classic::{
            chained_loops::ExtrusionLoop,
            entity_collections::{ExtrusionEntity, ExtrusionEntityCollection},
            materialize::ExtrusionRole,
        },
        seam_candidates::SeamCandidate,
    },
};

pub(super) fn place_layer(
    layer: &mut OrderedExtrusionLayer,
    plan: &alignment::LayerPlan,
    staggered: bool,
    scale: CoordinateScale,
) {
    let collections = layer
        .islands
        .iter_mut()
        .flat_map(|island| &mut island.entities)
        .filter_map(|entity| match entity {
            IslandPrintEntity::Perimeter(collection) => Some(collection),
            IslandPrintEntity::Fill(_)
            | IslandPrintEntity::FillCollection(_)
            | IslandPrintEntity::Thin(_) => None,
        });
    for (collection, perimeter_indices) in collections.zip(&plan.collection_perimeters) {
        place_collection(collection, perimeter_indices, plan, staggered, scale);
    }
}

fn place_collection(
    collection: &mut ExtrusionEntityCollection,
    perimeter_indices: &[usize],
    plan: &alignment::LayerPlan,
    staggered: bool,
    scale: CoordinateScale,
) {
    for (entity, &perimeter_index) in collection.entities.iter_mut().zip(perimeter_indices) {
        let ExtrusionEntity::Loop(ordered) = entity else {
            // GCode.cpp:6087–6100 routes open paths and MultiPaths to their
            // emitters; only extrude_loop invokes place_seam (:5773).
            continue;
        };
        let perimeter = &plan.candidates.perimeters[perimeter_index];
        let choice = &plan.choices[perimeter_index];
        let selected = choice.seam_index;
        ordered.unsupported_distance = plan
            .unsupported_distances
            .as_ref()
            .map(|values| values[selected]);
        let previous = if selected == perimeter.start_index {
            perimeter.end_index - 1
        } else {
            selected - 1
        };
        let next = if selected + 1 == perimeter.end_index {
            perimeter.start_index
        } else {
            selected + 1
        };
        let selected_candidate = &plan.candidates.points[selected];
        place_loop(
            &mut ordered.extrusion_loop,
            Placement {
                selected: selected_candidate,
                previous: &plan.candidates.points[previous],
                next: &plan.candidates.points[next],
                position: choice.final_position.unwrap_or_else(|| {
                    mesh::Vec3::new(
                        selected_candidate.position.x,
                        selected_candidate.position.y,
                        selected_candidate.position.z,
                    )
                }),
            },
            staggered,
            scale,
        );
    }
}

#[derive(Clone, Copy)]
struct Placement<'a> {
    selected: &'a SeamCandidate,
    previous: &'a SeamCandidate,
    next: &'a SeamCandidate,
    position: mesh::Vec3,
}

#[expect(
    clippy::approx_constant,
    reason = "OrcaSlicer uses the literal 1.4142 seam-depth coefficient"
)]
fn place_loop(
    loop_: &mut ExtrusionLoop,
    placement: Placement<'_>,
    staggered: bool,
    scale: CoordinateScale,
) {
    let selected = placement.selected.position;
    let mut seam = scale_position((placement.position.x, placement.position.y), scale);
    if loop_.paths[0].role == ExtrusionRole::Perimeter {
        let mut projection = closest_projection(&loop_.paths, seam, false);
        let mut depth = scale
            .unscale(seam.0 - projection.x)
            .hypot(scale.unscale(seam.1 - projection.y)) as f32;
        let angle = placement.selected.local_ccw_angle;
        let displacement = (
            placement.position.x - selected.x,
            placement.position.y - selected.y,
            placement.position.z - selected.z,
        );
        let displacement_squared = displacement.0.mul_add(
            displacement.0,
            displacement
                .1
                .mul_add(displacement.1, displacement.2 * displacement.2),
        );
        // `local_ccw_angle < -EPSILON` — upstream EPSILON is the shared
        // `1e-4` double literal (`libslic3r.h:52`), compared with the
        // float promoted to double (`SeamPlacer.cpp:1581`).
        if displacement_squared < depth && f64::from(angle) < -1e-4 {
            let previous = placement.previous.position;
            let next = placement.next.position;
            let to_previous = normalized((selected.x - previous.x, selected.y - previous.y));
            let to_next = normalized((selected.x - next.x, selected.y - next.y));
            let direction = (
                0.5 * (to_previous.0 + to_next.0),
                0.5 * (to_previous.1 + to_next.1),
            );
            depth = (1.4142 * f64::from(depth) / f64::from((angle * 0.5).cos())) as f32;
            seam = scale_position(
                (
                    selected.x + depth * direction.0,
                    selected.y + depth * direction.1,
                ),
                scale,
            );
            projection = closest_projection(&loop_.paths, seam, false);
        } else {
            // Convex corner: the perpendicular depth, not the distance to
            // the nearest point (upstream place_loop, SeamPlacer.cpp:1594).
            // `depth * beta_angle / 1.4142` multiplies in f32, promotes the
            // product to double for the division by the `1.4142` double
            // literal, then narrows back to float.
            depth = (f64::from(depth * (angle * 0.5).cos()) / 1.4142) as f32;
        }
        seam = (projection.x, projection.y);
        // Stagger inner seams inside the placement pass — upstream walks the
        // loop points from the projected position (SeamPlacer.cpp:1607-1620),
        // not a separate re-split pass.
        if staggered {
            seam = staggered_seam(loop_, projection, depth, scale);
        }
    }
    split_at(loop_, seam, scale);
}

/// Advance the projected seam along the loop (`SeamPlacer.cpp:1607-1620`).
fn staggered_seam(
    loop_: &ExtrusionLoop,
    mut point: super::split::Projection,
    mut depth: f32,
    scale: CoordinateScale,
) -> (i64, i64) {
    depth = loop_.paths[point.path].width.max(depth);
    while depth > 0.0 {
        let mut next = next_loop_point(loop_, &point);
        let a = (scale.unscale(point.x) as f32, scale.unscale(point.y) as f32);
        let b = (scale.unscale(next.x) as f32, scale.unscale(next.y) as f32);
        let dx = a.0 - b.0;
        let dy = a.1 - b.1;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist > depth {
            (next.x, next.y) = scale_position(
                (
                    a.0 + (b.0 - a.0) * depth / dist,
                    a.1 + (b.1 - a.1) * depth / dist,
                ),
                scale,
            );
        }
        depth -= dist;
        point = next;
    }
    (point.x, point.y)
}
