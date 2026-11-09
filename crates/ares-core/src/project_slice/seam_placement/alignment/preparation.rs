use std::collections::BTreeSet;

use crate::{
    ProcessSeamPosition,
    geometry::CoordinateScale,
    project_slice::{
        island_print_order::{IslandPrintEntity, OrderedExtrusionLayer},
        perimeters::{
            classic::{
                entity_collections::{ExtrusionEntity, ExtrusionEntityCollection},
                materialize::ExtrusionPath,
                traversal::PreparedPostClassicTraversal,
            },
            types::{PostPerimeterInputPrintObject, RegionLayerIndex},
        },
        seam_candidates::{self, LayerSeamCandidates, SeamPerimeter},
    },
};

use super::{
    LayerPlan, PerimeterChoice, context,
    contour_continuation::{Contour, Layers},
    is_better,
};
use crate::project_slice::seam_placement::{candidate_penalty, visibility};

pub(in crate::project_slice::seam_placement) fn prepare(
    layers: (&[OrderedExtrusionLayer], &[f32]),
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    nozzle_diameter: f32,
    visibility: Option<&visibility::GlobalVisibility>,
) -> Vec<LayerPlan> {
    let (layers, layer_zs) = layers;
    let placement = traversal
        .print_object_config(object_index)
        .0
        .object
        .seam_position;
    let input = &traversal.objects[object_index]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object;
    let max_nozzle_diameter = traversal
        .resolved
        .views
        .full
        .project
        .print
        .nozzle_diameter
        .0
        .iter()
        .map(|value| value.0)
        .fold(0.0_f64, f64::max);
    let mut plans = layers
        .iter()
        .zip(layer_zs)
        .enumerate()
        .map(|(index, (layer, &z))| {
            let regions = layer_regions(layer, input, index);
            prepare_layer(
                &regions,
                z,
                traversal.scale,
                (nozzle_diameter, max_nozzle_diameter),
                (visibility, placement),
            )
        })
        .collect::<Vec<_>>();
    let (post_region, layer_slices) = input.object.as_parts();
    let (planned_object, _, _) = post_region.as_parts();
    context::populate(
        &mut plans,
        &embedding_layers(layers),
        layer_slices,
        &planned_object.layers,
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

// SeamPlacer.cpp:1915-1929: immutable geometry, independent of choices.
pub(super) fn contours(layers: &[LayerPlan]) -> Layers {
    layers
        .iter()
        .map(|layer| {
            layer
                .candidates
                .perimeters
                .iter()
                .map(|perimeter| Contour {
                    points: layer.positions[perimeter.start_index..perimeter.end_index]
                        .iter()
                        .map(|point| [f64::from(point.x), f64::from(point.y)])
                        .collect(),
                    hole: perimeter.is_hole,
                    reliable: perimeter.contour_reliable,
                    width: f64::from(perimeter.flow_width),
                    nozzle: layer.max_nozzle_diameter,
                    min: [0.0; 2],
                    max: [0.0; 2],
                })
                .collect()
        })
        .collect()
}

fn embedding_layers(layers: &[OrderedExtrusionLayer]) -> Vec<bool> {
    layers
        .iter()
        .map(|layer| {
            layer
                .islands
                .iter()
                .flat_map(|island| island.perimeter_regions.iter().copied())
                .collect::<BTreeSet<_>>()
                .len()
                > 1
        })
        .collect()
}

fn layer_regions<'a>(
    layer: &'a OrderedExtrusionLayer,
    input: &PostPerimeterInputPrintObject,
    layer_index: usize,
) -> Vec<seam_candidates::RegionPerimeters<'a>> {
    layer
        .islands
        .iter()
        .flat_map(|island| {
            island
                .entities
                .iter()
                .filter_map(|entity| match entity {
                    IslandPrintEntity::Perimeter(collection) => Some(collection),
                    IslandPrintEntity::Fill(_)
                    | IslandPrintEntity::FillCollection(_)
                    | IslandPrintEntity::Thin(_) => None,
                })
                .zip(&island.perimeter_regions)
        })
        .map(|(collection, &region_index)| {
            let record_index = input.region_layer_record_index(RegionLayerIndex {
                region_index,
                layer_index,
            });
            let record = input.records[record_index]
                .as_ref()
                .expect("printing perimeter region has a flow record");
            seam_candidates::RegionPerimeters {
                collection: Some(collection),
                external_flow_width: record.ext_perimeter_flow.width,
            }
        })
        .collect()
}

fn prepare_layer(
    regions: &[seam_candidates::RegionPerimeters<'_>],
    z: f32,
    scale: CoordinateScale,
    (nozzle_diameter, max_nozzle_diameter): (f32, f64),
    (visibility, placement): (Option<&visibility::GlobalVisibility>, ProcessSeamPosition),
) -> LayerPlan {
    let collections = regions
        .iter()
        .map(|region| region.collection.expect("printing perimeter collection"))
        .collect::<Vec<_>>();
    let mut source_order = (0..collections.len()).collect::<Vec<_>>();
    source_order.sort_unstable_by_key(|&index| collections[index].source_order);
    let regions = source_order
        .iter()
        .map(|&index| seam_candidates::RegionPerimeters {
            collection: Some(collections[index]),
            external_flow_width: regions[index].external_flow_width,
        })
        .collect::<Vec<_>>();
    let candidates = seam_candidates::generate_regions(&regions, z, scale, nozzle_diameter);
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
    let association = PerimeterAssociation {
        candidates: &candidates,
        positions: &positions,
        point_tree: &point_tree,
        z,
        scale,
    };
    let collection_perimeters = collections
        .iter()
        .map(|collection| association.collection_map(collection))
        .collect();
    LayerPlan {
        placement,
        candidates,
        choices: Vec::new(),
        collection_perimeters,
        scores,
        z,
        max_nozzle_diameter,
        overhangs: vec![0.0; candidate_count],
        unsupported_distances: None,
        embedded_distances: vec![0.0; candidate_count],
        positions,
        point_tree,
    }
}

struct PerimeterAssociation<'a> {
    candidates: &'a LayerSeamCandidates,
    positions: &'a [super::Vec3],
    point_tree: &'a super::PointKdTree,
    z: f32,
    scale: CoordinateScale,
}

impl PerimeterAssociation<'_> {
    fn collection_map(&self, collection: &ExtrusionEntityCollection) -> Vec<usize> {
        collection
            .entities
            .iter()
            .map(|entity| {
                let paths = match entity {
                    ExtrusionEntity::Loop(ordered) => &ordered.extrusion_loop.paths,
                    ExtrusionEntity::MultiPath(multi_path) => &multi_path.paths,
                    ExtrusionEntity::Path(path) => std::slice::from_ref(path),
                };
                self.closest_perimeter(paths)
            })
            .collect()
    }

    fn closest_perimeter(&self, paths: &[ExtrusionPath]) -> usize {
        let point_count = paths.iter().map(|path| path.polyline.points.len()).sum();
        let mut path_index = 0;
        let mut point_index = 0;
        let mut candidate_index = 0;
        let mut closest_perimeter = None;
        for _ in 0..point_count {
            let point = paths[path_index].polyline.points[point_index];
            candidate_index = self.point_tree.closest(
                self.positions,
                super::Vec3::new(
                    self.scale.unscale(point.x) as f32,
                    self.scale.unscale(point.y) as f32,
                    self.z,
                ),
            );
            let perimeter_index = self.candidates.points[candidate_index].perimeter_index;
            if closest_perimeter == Some(perimeter_index) {
                break;
            }
            closest_perimeter = Some(perimeter_index);
            point_index += 1;
            if point_index == paths[path_index].polyline.points.len() {
                path_index = (path_index + 1) % paths.len();
                point_index = 0;
            }
        }
        self.candidates.points[candidate_index].perimeter_index
    }
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
