mod alignment;
mod fitting;
mod mesh;
mod runtime;
mod sampling;
mod spatial;
mod spline;
mod split;
mod visibility;

use crate::{
    ProcessSeamPosition,
    geometry::CoordinateScale,
    project_slice::{
        island_print_order::{
            IslandPrintEntity, NearestSeamLayer, OrderedExtrusionLayer,
            PreparedPostIslandPrintOrder,
        },
        perimeters::classic::{
            chained_loops::ExtrusionLoop,
            entity_collections::{ExtrusionEntity, ExtrusionEntityCollection},
            materialize::{ExtrusionPath, ExtrusionRole, Point3},
            traversal::PreparedPostClassicTraversal,
        },
        seam_candidates::SeamCandidate,
    },
};
use split::{closest_projection, next_loop_point, normalized, scale_position, split_at};

const VISIBILITY_SAMPLE_COUNT: usize = 30_000;
const ANGLE_IMPORTANCE_ALIGNED: f32 = 0.6;
/// `SeamPlacer.hpp:126` — angle weight for the spNearest comparator.
const ANGLE_IMPORTANCE_NEAREST: f32 = 1.0;

/// `SeamPlacer.cpp:44-49` — `gauss(value, mean, falloff)` used for the
/// emit-time distance penalty (`cpp:784-785`).
fn gauss_penalty(value: f32, falloff_speed: f32) -> f32 {
    let denominator = falloff_speed * value * value + 1.0;
    let exponent = 1.0 / denominator;
    (std::f32::consts::E.exp() - 1.0).recip() * (exponent.exp() - 1.0)
}

#[cfg(test)]
use runtime::is_closed_axis_rectangle;
pub(in crate::project_slice) use runtime::{
    place_nearest, place_nearest_penalized, place_nearest_projection,
};

pub(in crate::project_slice) fn apply(prepared: &mut PreparedPostIslandPrintOrder) {
    let predecessor = &prepared.predecessor;
    let traversal = &predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let placements = runtime::placement_modes(&traversal.resolved.objects, &prepared.objects);
    if !placements.iter().any(Option::is_some) {
        return;
    }

    let mesh = mesh::TriangleMesh::from_project(&traversal.project);
    if mesh.triangles.is_empty() {
        return;
    }
    let front_bias = placements.iter().any(|placement| {
        placement.is_some_and(|mode| mode == crate::ProcessSeamPosition::AlignedBack)
    });
    let visibility =
        visibility::GlobalVisibility::from_mesh(mesh, VISIBILITY_SAMPLE_COUNT, front_bias);
    let nozzle_diameter = traversal
        .resolved
        .views
        .full
        .project
        .print
        .nozzle_diameter
        .0
        .first()
        .map_or(0.4, |value| value.0) as f32;
    let mut nearest_seam_plans = std::mem::take(&mut prepared.nearest_seam_plans);
    nearest_seam_plans.resize_with(placements.len(), Vec::new);
    apply_objects(
        &mut prepared.objects,
        traversal,
        &placements,
        &visibility,
        nozzle_diameter,
        &mut nearest_seam_plans,
    );
    prepared.nearest_seam_plans = nearest_seam_plans;
}

fn apply_objects(
    objects: &mut [Vec<OrderedExtrusionLayer>],
    traversal: &PreparedPostClassicTraversal,
    placements: &[Option<ProcessSeamPosition>],
    visibility: &visibility::GlobalVisibility,
    nozzle_diameter: f32,
    nearest_plans: &mut [Vec<NearestSeamLayer>],
) {
    for (object_index, layers) in objects.iter_mut().enumerate() {
        let Some(placement) = placements[object_index] else {
            continue;
        };
        let layer_zs = runtime::layer_mid_zs(&traversal.objects[object_index].records);
        let mut plans = alignment::prepare(
            (layers, &layer_zs),
            traversal,
            object_index,
            nozzle_diameter,
            visibility,
        );
        match placement {
            ProcessSeamPosition::Aligned | ProcessSeamPosition::AlignedBack => {
                alignment::align(&mut plans)
            }
            ProcessSeamPosition::Random => alignment::randomize(&mut plans),
            ProcessSeamPosition::Nearest => {
                // `SeamPlacer.cpp:1465` skips `pick_seam_point` for spNearest —
                // the seam is picked at emit with actual nozzle position
                // (`place_seam` cpp:1500-1560 via
                // `pick_nearest_seam_point_index` cpp:930-940). Store the
                // penalty data for the emit-time selection; no loop splitting
                // here.
                nearest_plans[object_index] = nearest_seam_data(&plans, visibility);
                continue;
            }
            _ => unreachable!("only active seam placement modes are prepared"),
        }
        let staggered = traversal.resolved.objects[object_index]
            .object
            .staggered_inner_seams
            .0;
        for (layer, plan) in layers.iter_mut().zip(&plans) {
            place_layer(layer, plan, staggered, traversal.scale);
        }
    }
}

fn nearest_seam_data(
    plans: &[alignment::LayerPlan],
    visibility: &visibility::GlobalVisibility,
) -> Vec<NearestSeamLayer> {
    plans
        .iter()
        .map(|plan| {
            let scores = plan
                .candidates
                .points
                .iter()
                .map(|candidate| {
                    let position = mesh::Vec3::new(
                        candidate.position.x,
                        candidate.position.y,
                        candidate.position.z,
                    );
                    visibility.at(position)
                        + ANGLE_IMPORTANCE_NEAREST * angle_penalty(candidate.local_ccw_angle)
                })
                .collect();
            nearest_seam_layer(plan, scores)
        })
        .collect()
}

fn nearest_seam_layer(plan: &alignment::LayerPlan, scores: Vec<f32>) -> NearestSeamLayer {
    NearestSeamLayer {
        positions: plan
            .candidates
            .points
            .iter()
            .map(|candidate| (candidate.position.x, candidate.position.y))
            .collect(),
        scores,
        overhangs: plan.overhangs.clone(),
        perimeter_ranges: plan
            .candidates
            .perimeters
            .iter()
            .map(|perimeter| (perimeter.start_index, perimeter.end_index))
            .collect(),
        perimeter_of_candidate: plan
            .candidates
            .points
            .iter()
            .map(|candidate| candidate.perimeter_index)
            .collect(),
        ccw_angles: plan
            .candidates
            .points
            .iter()
            .map(|candidate| candidate.local_ccw_angle)
            .collect(),
    }
}

fn place_layer(
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
            // Aligned-seam placement targets loops (`SeamPlacer.cpp:1500`);
            // arachne wall generation stays typed-rejected before
            // materialization, so no multi-path can reach placement.
            unreachable!("multi-path seam placement lands with the arachne materialization seam");
        };
        let perimeter = &plan.candidates.perimeters[perimeter_index];
        let choice = &plan.choices[perimeter_index];
        let selected = choice.seam_index;
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

fn candidate_penalty(candidate: &SeamCandidate, visibility: &visibility::GlobalVisibility) -> f32 {
    let position = mesh::Vec3::new(
        candidate.position.x,
        candidate.position.y,
        candidate.position.z,
    );
    visibility.at(position) + ANGLE_IMPORTANCE_ALIGNED * angle_penalty(candidate.local_ccw_angle)
}

fn angle_penalty(angle: f32) -> f32 {
    let denominator = 3.0 * angle * angle + 1.0;
    let gaussian = ((1.0 / denominator).exp() - 1.0) / (1.0_f32.exp() - 1.0);
    gaussian + 1.0 / (2.0 + (-angle).exp())
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
        let mut projection = closest_projection(&loop_.paths, seam);
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
        if displacement_squared < depth && angle < -f32::EPSILON {
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
            projection = closest_projection(&loop_.paths, seam);
        } else {
            // Convex corner: the perpendicular depth, not the distance to
            // the nearest point (upstream place_loop, SeamPlacer.cpp:1594).
            depth *= (angle * 0.5).cos() / 1.4142;
        }
        seam = (projection.x, projection.y);
        // Stagger inner seams inside the placement pass — upstream walks the
        // loop points from the projected position (SeamPlacer.cpp:1607-1620),
        // not a separate re-split pass.
        if staggered {
            let mut point = projection;
            depth = loop_.paths[point.path].width.max(depth);
            while depth > 0.0 {
                let mut next = next_loop_point(loop_, &point);
                let a = (point.x, point.y);
                let b = (next.x, next.y);
                let dist = scale.unscale(a.0 - b.0).hypot(scale.unscale(a.1 - b.1)) as f32;
                if dist > depth {
                    let ratio = depth / dist;
                    next.x = (a.0 as f64 + f64::from(ratio) * (b.0 - a.0) as f64) as i64;
                    next.y = (a.1 as f64 + f64::from(ratio) * (b.1 - a.1) as f64) as i64;
                }
                depth -= dist;
                point = next;
            }
            seam = (point.x, point.y);
        }
    }
    if let Ok(path) = std::env::var("ARES_DUMP_SEAMPT") {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(
                file,
                "SPT {} {}",
                scale.unscale(seam.0),
                scale.unscale(seam.1)
            );
        }
    }
    split_at(loop_, seam, scale);
}

#[cfg(test)]
mod tests;
