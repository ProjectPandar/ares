mod alignment;
pub(in crate::project_slice) mod fitting;
pub(in crate::project_slice) mod kdtree;
mod mesh;
pub(super) mod paint;
mod placement;
mod runtime;
mod sampling;
mod short_edge_collapse;
mod spatial;
mod spline;
mod split;
mod visibility;

use crate::{
    ProcessSeamPosition, SliceError,
    project_slice::{
        island_print_order::{
            NearestSeamLayer, OrderedExtrusionLayer, PreparedPostIslandPrintOrder,
        },
        perimeters::classic::traversal::PreparedPostClassicTraversal,
        seam_candidates::SeamCandidate,
    },
};

const VISIBILITY_SAMPLE_COUNT: usize = 30_000;
const ANGLE_IMPORTANCE_ALIGNED: f32 = 0.6;
/// `SeamPlacer.hpp:126` — angle weight for the spNearest comparator.
const ANGLE_IMPORTANCE_NEAREST: f32 = 1.0;

/// `SeamPlacer.cpp:44-49` — `gauss(value, mean, falloff)` used for the
/// emit-time distance penalty (`cpp:784-785`).
fn gauss_penalty(value: f32, falloff_speed: f32) -> f32 {
    let denominator = falloff_speed * value * value + 1.0;
    let exponent = 1.0 / denominator;
    let exponential = f64::from(exponent).exp() as f32;
    let normalization = 1.0_f64.exp() as f32;
    (exponential - 1.0) / (normalization - 1.0)
}

pub(in crate::project_slice) use runtime::{
    place_nearest, place_nearest_penalized, place_nearest_projection,
};

pub(super) use paint::validate_paint;

pub(in crate::project_slice) fn apply(
    prepared: &mut PreparedPostIslandPrintOrder,
) -> Result<(), SliceError> {
    let predecessor = &prepared.predecessor;
    let traversal = &predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let configurations = (0..traversal.objects.len())
        .map(|index| traversal.print_object_config(index).0)
        .collect::<Vec<_>>();
    let placements = runtime::placement_modes(
        &configurations,
        traversal.resolved.views.full.process.print.spiral_mode.0,
    );
    if !placements.iter().any(Option::is_some) {
        return Ok(());
    }

    // `SeamPlacer.cpp:1442` — occlusion is computed per PrintObject, each
    // with its own `trafo_centered` frame; share one visibility model per
    // object index.
    let visibilities = placements
        .iter()
        .enumerate()
        .map(|(object_index, placement)| {
            // SeamPlacer.cpp:1440-1457 leaves rear/random visibility at zero.
            if !matches!(
                placement,
                Some(
                    ProcessSeamPosition::Aligned
                        | ProcessSeamPosition::Nearest
                        | ProcessSeamPosition::AlignedBack
                )
            ) {
                return None;
            }
            let source_index = traversal
                .print_object_config(object_index)
                .0
                .source_object_index;
            let mesh = mesh::TriangleMesh::from_object(
                &traversal.project.objects()[source_index],
                traversal.print_object_config(object_index).1,
                traversal.scale,
            );
            (!mesh.triangles.is_empty()).then(|| {
                let front_bias =
                    placement.is_some_and(|mode| mode == crate::ProcessSeamPosition::AlignedBack);
                visibility::GlobalVisibility::from_mesh(mesh, VISIBILITY_SAMPLE_COUNT, front_bias)
            })
        })
        .collect::<Vec<_>>();
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
        SeamObjectSettings {
            placements: &placements,
            visibilities: &visibilities,
        },
        nozzle_diameter,
        &mut nearest_seam_plans,
    )?;
    prepared.nearest_seam_plans = nearest_seam_plans;
    Ok(())
}

/// Inputs from `SeamPlacer.cpp::SeamPlacer::init object choices and visibility`.
struct SeamObjectSettings<'a> {
    placements: &'a [Option<ProcessSeamPosition>],
    visibilities: &'a [Option<visibility::GlobalVisibility>],
}

fn apply_objects(
    objects: &mut [Vec<OrderedExtrusionLayer>],
    traversal: &PreparedPostClassicTraversal,
    SeamObjectSettings {
        placements,
        visibilities,
    }: SeamObjectSettings<'_>,
    nozzle_diameter: f32,
    nearest_plans: &mut [Vec<NearestSeamLayer>],
) -> Result<(), SliceError> {
    for (object_index, layers) in objects.iter_mut().enumerate() {
        let Some(placement) = placements[object_index] else {
            continue;
        };
        let visibility = visibilities[object_index].as_ref();
        if visibility.is_none()
            && !matches!(
                placement,
                ProcessSeamPosition::Back | ProcessSeamPosition::Random
            )
        {
            continue;
        }
        // SeamPlacer.cpp:1024-1032 consumes each physical Layer::slice_z.
        let layer_zs = traversal.objects[object_index]
            .physical_layers()
            .iter()
            .map(|layer| layer.slice_z as f32)
            .collect::<Vec<_>>();
        let (source, group) = traversal.print_object_config(object_index);
        let paint = paint::SeamPaint::from_object(
            &traversal.project.objects()[source.source_object_index],
            group,
            traversal.scale,
        )?;
        let mut plans = alignment::prepare(
            (layers, &layer_zs),
            traversal,
            object_index,
            nozzle_diameter,
            (visibility, paint.as_ref()),
        )?;
        match placement {
            ProcessSeamPosition::Aligned
            | ProcessSeamPosition::AlignedBack
            | ProcessSeamPosition::Back => alignment::align(&mut plans),
            ProcessSeamPosition::Random => alignment::randomize(&mut plans),
            ProcessSeamPosition::Nearest => {
                // `SeamPlacer.cpp:1465` skips `pick_seam_point` for spNearest —
                // the seam is picked at emit with actual nozzle position
                // (`place_seam` cpp:1500-1560 via
                // `pick_nearest_seam_point_index` cpp:930-940). Store the
                // penalty data for the emit-time selection; no loop splitting
                // here.
                nearest_plans[object_index] = nearest_seam_data(&plans, visibility.unwrap());
                continue;
            }
        }
        let staggered = traversal
            .print_object_config(object_index)
            .0
            .object
            .staggered_inner_seams
            .0;
        for (layer, plan) in layers.iter_mut().zip(&plans) {
            placement::place_layer(layer, plan, staggered, traversal.scale);
        }
    }
    Ok(())
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
            .map(|candidate| {
                (
                    candidate.position.x,
                    candidate.position.y,
                    candidate.position.z,
                )
            })
            .collect(),
        scores,
        paint_states: plan
            .candidates
            .points
            .iter()
            .map(|candidate| candidate.paint)
            .collect(),
        overhangs: plan.overhangs.clone(),
        unsupported_distances: plan.unsupported_distances.clone(),
        embedded: plan.embedded_distances.clone(),
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
        kdtree: std::cell::OnceCell::new(),
    }
}

fn candidate_penalty(
    candidate: &SeamCandidate,
    visibility: Option<&visibility::GlobalVisibility>,
) -> f32 {
    let visibility = visibility.map_or(0.0, |visibility| {
        let position = mesh::Vec3::new(
            candidate.position.x,
            candidate.position.y,
            candidate.position.z,
        );
        visibility.at(position)
    });
    visibility + ANGLE_IMPORTANCE_ALIGNED * angle_penalty(candidate.local_ccw_angle)
}

fn angle_penalty(angle: f32) -> f32 {
    let denominator = 3.0 * angle * angle + 1.0;
    // Round exp once to the upstream float: native expf and WASM expf
    // differ by an ulp for real seam weights (SeamPlacer.cpp:42-59).
    let exponent = f64::from(1.0 / denominator).exp() as f32;
    let sigmoid_exponent = f64::from(-angle).exp() as f32;
    let gaussian = (exponent - 1.0) / (1.0_f32.exp() - 1.0);
    gaussian + 1.0 / (2.0 + sigmoid_exponent)
}
