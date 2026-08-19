use crate::{
    geometry::CoordinateScale,
    project_slice::{
        island_print_order::{IslandPrintEntity, OrderedExtrusionLayer},
        perimeters::classic::{
            entity_collections::ExtrusionEntityCollection, materialize::ExtrusionRole,
            traversal::PreparedPostClassicTraversal,
        },
        seam_candidates::{self, LayerSeamCandidates, SeamPerimeter},
    },
};

use super::{LayerPlan, PerimeterChoice, context, is_better};
use crate::project_slice::seam_placement::{candidate_penalty, visibility};

pub(in crate::project_slice::seam_placement) fn prepare(
    layers: (&[OrderedExtrusionLayer], &[f32]),
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    nozzle_diameter: f32,
    visibility: &visibility::GlobalVisibility,
) -> Vec<LayerPlan> {
    let (layers, layer_zs) = layers;
    let mut plans = layers
        .iter()
        .zip(layer_zs)
        .map(|(layer, &z)| prepare_layer(layer, z, traversal.scale, nozzle_diameter, visibility))
        .collect::<Vec<_>>();
    let input = &traversal.objects[object_index]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object;
    let layer_slices = input.object.as_parts().1;
    context::populate(
        &mut plans,
        &embedding_layers(traversal, object_index, layers.len()),
        layer_slices,
        traversal.scale,
    );
    for plan in &mut plans {
        plan.choices = plan
            .candidates
            .perimeters
            .iter()
            .map(|perimeter| PerimeterChoice {
                seam_index: best_candidate(perimeter, plan),
                final_position: None,
                finalized: false,
            })
            .collect();
    }
    plans
}

fn embedding_layers(
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    layer_count: usize,
) -> Vec<bool> {
    let object = &traversal.objects[object_index];
    let input = &object
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object;
    let mut region_counts = vec![0_u16; layer_count];
    for (input, perimeters) in input.records.iter().zip(&object.records) {
        let (Some(input), Some(perimeters)) = (input, perimeters) else {
            continue;
        };
        if perimeters
            .surfaces
            .iter()
            .any(|surface| !surface.roots.is_empty())
        {
            region_counts[input.planned_layer_index] += 1;
        }
    }
    region_counts.into_iter().map(|count| count > 1).collect()
}

fn prepare_layer(
    layer: &OrderedExtrusionLayer,
    z: f32,
    scale: CoordinateScale,
    nozzle_diameter: f32,
    visibility: &visibility::GlobalVisibility,
) -> LayerPlan {
    let collections = layer
        .islands
        .iter()
        .flat_map(|island| &island.entities)
        .filter_map(|entity| match entity {
            IslandPrintEntity::Perimeter(collection) => Some(collection),
            IslandPrintEntity::Fill(_) | IslandPrintEntity::Thin(_) => None,
        });
    let mut candidates = LayerSeamCandidates::default();
    let mut collection_perimeters = Vec::new();
    for collection in collections {
        let external_flow_width = collection
            .entities
            .iter()
            .flat_map(|entity| &entity.extrusion_loop.paths)
            .find(|path| path.role == ExtrusionRole::ExternalPerimeter)
            .map_or(0.0, |path| path.width);
        let local = seam_candidates::generate_regions(
            &[seam_candidates::RegionPerimeters {
                collections: std::slice::from_ref(collection),
                external_flow_width,
            }],
            z,
            scale,
            nozzle_diameter,
        );
        let perimeter_base = candidates.perimeters.len();
        let point_base = candidates.points.len();
        collection_perimeters.push(collection_perimeter_map(
            collection,
            &local,
            scale,
            perimeter_base,
        ));
        candidates
            .points
            .extend(local.points.into_iter().map(|mut candidate| {
                candidate.perimeter_index += perimeter_base;
                candidate
            }));
        candidates
            .perimeters
            .extend(local.perimeters.into_iter().map(|perimeter| SeamPerimeter {
                start_index: perimeter.start_index + point_base,
                end_index: perimeter.end_index + point_base,
                flow_width: perimeter.flow_width,
            }));
    }
    let candidate_count = candidates.points.len();
    let scores = candidates
        .points
        .iter()
        .map(|candidate| candidate_penalty(candidate, visibility))
        .collect::<Vec<_>>();
    let positions = candidates
        .points
        .iter()
        .map(|candidate| {
            let position = candidate.position;
            super::Vec3::new(position.x, position.y, position.z)
        })
        .collect::<Vec<_>>();
    let point_tree = super::PointKdTree::new(&positions);
    LayerPlan {
        candidates,
        choices: Vec::new(),
        collection_perimeters,
        scores,
        z,
        overhangs: vec![0.0; candidate_count],
        embedded_distances: vec![0.0; candidate_count],
        positions,
        point_tree,
    }
}

fn collection_perimeter_map(
    collection: &ExtrusionEntityCollection,
    candidates: &LayerSeamCandidates,
    scale: CoordinateScale,
    perimeter_base: usize,
) -> Vec<usize> {
    collection
        .entities
        .iter()
        .map(|entity| {
            let point = entity
                .extrusion_loop
                .paths
                .first()
                .and_then(|path| path.polyline.candidate_points().first())
                .expect("an extrusion loop has a point");
            let target = (scale.unscale(point.x) as f32, scale.unscale(point.y) as f32);
            perimeter_base
                + candidates
                    .perimeters
                    .iter()
                    .enumerate()
                    .min_by(|(_, left), (_, right)| {
                        perimeter_distance_squared(left, &candidates.points, target).total_cmp(
                            &perimeter_distance_squared(right, &candidates.points, target),
                        )
                    })
                    .expect("a collection has a seam perimeter")
                    .0
        })
        .collect()
}

fn perimeter_distance_squared(
    perimeter: &SeamPerimeter,
    candidates: &[seam_candidates::SeamCandidate],
    point: (f32, f32),
) -> f32 {
    candidates[perimeter.start_index..perimeter.end_index]
        .iter()
        .map(|candidate| {
            let x = candidate.position.x - point.0;
            let y = candidate.position.y - point.1;
            x.mul_add(x, y * y)
        })
        .reduce(f32::min)
        .expect("a seam perimeter has candidates")
}

fn best_candidate(perimeter: &SeamPerimeter, layer: &LayerPlan) -> usize {
    let mut best = perimeter.start_index;
    for index in perimeter.start_index + 1..perimeter.end_index {
        if is_better(layer, index, best) {
            best = index;
        }
    }
    best
}
