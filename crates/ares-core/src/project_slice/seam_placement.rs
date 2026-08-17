mod alignment;
mod mesh;
mod sampling;
mod spatial;
mod spline;
mod visibility;

use crate::{
    ProcessSeamPosition,
    geometry::CoordinateScale,
    project_slice::{
        gcode_emit,
        island_print_order::{
            IslandPrintEntity, OrderedExtrusionLayer, PreparedPostIslandPrintOrder,
        },
        perimeters::classic::{
            chained_loops::ExtrusionLoop,
            entity_collections::ExtrusionEntityCollection,
            materialize::{ExtrusionPath, ExtrusionRole, Point3, Polyline3},
            traversal::PreparedPostClassicTraversal,
        },
        seam_candidates::SeamCandidate,
    },
};

const VISIBILITY_SAMPLE_COUNT: usize = 30_000;
const ANGLE_IMPORTANCE_ALIGNED: f32 = 0.6;

pub(in crate::project_slice) fn apply(prepared: &mut PreparedPostIslandPrintOrder) {
    let predecessor = &prepared.predecessor;
    let traversal = &predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let aligned = traversal
        .resolved
        .objects
        .iter()
        .map(|object| object.object.seam_position == ProcessSeamPosition::Aligned)
        .collect::<Vec<_>>();
    if !aligned.iter().any(|&value| value) {
        return;
    }
    let center = gcode_emit::footprint::model_center(traversal).unwrap_or_default();
    let mesh = mesh::TriangleMesh::from_project(&traversal.project, center);
    if mesh.triangles.is_empty() {
        return;
    }
    let visibility = visibility::GlobalVisibility::from_mesh(mesh, VISIBILITY_SAMPLE_COUNT);
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
    apply_objects(
        &mut prepared.objects,
        traversal,
        &aligned,
        &visibility,
        nozzle_diameter,
    );
}

fn apply_objects(
    objects: &mut [Vec<OrderedExtrusionLayer>],
    traversal: &PreparedPostClassicTraversal,
    aligned: &[bool],
    visibility: &visibility::GlobalVisibility,
    nozzle_diameter: f32,
) {
    for (object_index, layers) in objects.iter_mut().enumerate() {
        if !aligned[object_index] {
            continue;
        }
        let layer_zs = traversal.objects[object_index]
            .records
            .iter()
            .scan(0.0_f32, |print_z, record| {
                let height = record.as_ref().map_or(0.0, |record| record.layer_height) as f32;
                *print_z += height;
                Some(*print_z - 0.5 * height)
            })
            .collect::<Vec<_>>();
        let mut plans = alignment::prepare(
            (layers, &layer_zs),
            traversal,
            object_index,
            nozzle_diameter,
            visibility,
        );
        alignment::align(&mut plans);
        for (layer, plan) in layers.iter_mut().zip(&plans) {
            place_layer(layer, plan, traversal.scale);
        }
    }
}

fn place_layer(
    layer: &mut OrderedExtrusionLayer,
    plan: &alignment::LayerPlan,
    scale: CoordinateScale,
) {
    let collections = layer
        .islands
        .iter_mut()
        .flat_map(|island| &mut island.entities)
        .filter_map(|entity| match entity {
            IslandPrintEntity::Perimeter(collection) => Some(collection),
            IslandPrintEntity::Fill(_) | IslandPrintEntity::Thin(_) => None,
        });
    for (collection, perimeter_indices) in collections.zip(&plan.collection_perimeters) {
        place_collection(collection, perimeter_indices, plan, scale);
    }
}

fn place_collection(
    collection: &mut ExtrusionEntityCollection,
    perimeter_indices: &[usize],
    plan: &alignment::LayerPlan,
    scale: CoordinateScale,
) {
    for (entity, &perimeter_index) in collection.entities.iter_mut().zip(perimeter_indices) {
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
            &mut entity.extrusion_loop,
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
fn place_loop(loop_: &mut ExtrusionLoop, placement: Placement<'_>, scale: CoordinateScale) {
    let selected = placement.selected.position;
    let mut seam = (
        f64::from(placement.position.x),
        f64::from(placement.position.y),
    );
    if loop_.paths[0].role == ExtrusionRole::Perimeter {
        let projection = closest_projection(&loop_.paths, seam, scale);
        let mut depth = projection.distance;
        let angle = placement.selected.local_ccw_angle;
        let displacement_squared = f64::from(placement.position.x - selected.x).mul_add(
            f64::from(placement.position.x - selected.x),
            f64::from(placement.position.y - selected.y)
                * f64::from(placement.position.y - selected.y),
        );
        if displacement_squared < depth && angle < -f32::EPSILON {
            let previous = placement.previous.position;
            let next = placement.next.position;
            let to_previous = normalized((
                f64::from(selected.x - previous.x),
                f64::from(selected.y - previous.y),
            ));
            let to_next = normalized((
                f64::from(selected.x - next.x),
                f64::from(selected.y - next.y),
            ));
            let direction = (
                0.5 * (to_previous.0 + to_next.0),
                0.5 * (to_previous.1 + to_next.1),
            );
            depth = 1.4142 * depth / f64::from((angle * 0.5).cos());
            seam.0 = f64::from(selected.x) + depth * direction.0;
            seam.1 = f64::from(selected.y) + depth * direction.1;
        }
        let projection = closest_projection(&loop_.paths, seam, scale);
        seam = (scale.unscale(projection.x), scale.unscale(projection.y));
    }
    split_at(loop_, seam, scale);
}

fn normalized(vector: (f64, f64)) -> (f64, f64) {
    let length = vector.0.hypot(vector.1);
    (vector.0 / length, vector.1 / length)
}

struct Projection {
    path: usize,
    segment: usize,
    x: i64,
    y: i64,
    distance: f64,
}

fn closest_projection(
    paths: &[ExtrusionPath],
    target: (f64, f64),
    scale: CoordinateScale,
) -> Projection {
    let mut best = None::<Projection>;
    for (path_index, path) in paths.iter().enumerate() {
        for (segment_index, segment) in path.polyline.points.windows(2).enumerate() {
            let a = (scale.unscale(segment[0].x), scale.unscale(segment[0].y));
            let b = (scale.unscale(segment[1].x), scale.unscale(segment[1].y));
            let edge = (b.0 - a.0, b.1 - a.1);
            let length_squared = edge.0.mul_add(edge.0, edge.1 * edge.1);
            let ratio = if length_squared == 0.0 {
                0.0
            } else {
                ((target.0 - a.0) * edge.0 + (target.1 - a.1) * edge.1) / length_squared
            }
            .clamp(0.0, 1.0);
            let point = (a.0 + ratio * edge.0, a.1 + ratio * edge.1);
            let distance = (target.0 - point.0).hypot(target.1 - point.1);
            if best.as_ref().is_none_or(|best| distance < best.distance) {
                best = Some(Projection {
                    path: path_index,
                    segment: segment_index,
                    x: (point.0 / scale.factor()).round() as i64,
                    y: (point.1 / scale.factor()).round() as i64,
                    distance,
                });
            }
        }
    }
    best.expect("an extrusion loop has a segment")
}

fn split_at(loop_: &mut ExtrusionLoop, seam: (f64, f64), scale: CoordinateScale) {
    let projection = closest_projection(&loop_.paths, seam, scale);
    let mut paths = std::mem::take(&mut loop_.paths);
    let following = paths.split_off(projection.path + 1);
    let target = paths.pop().expect("projected path exists");
    let split = projected_parts(target, &projection);
    loop_.paths.extend(split.0);
    loop_.paths.extend(following);
    loop_.paths.extend(paths);
    loop_.paths.extend(split.1);
}

fn projected_parts(
    path: ExtrusionPath,
    projection: &Projection,
) -> (Option<ExtrusionPath>, Option<ExtrusionPath>) {
    let point = Point3 {
        x: projection.x,
        y: projection.y,
        z: path.polyline.points[projection.segment].z,
    };
    let source = &path.polyline.points;
    let start = source[projection.segment];
    let end = source[projection.segment + 1];
    let (suffix, prefix) = if point.x == start.x && point.y == start.y {
        (
            source[projection.segment..].to_vec(),
            source[..=projection.segment].to_vec(),
        )
    } else if point.x == end.x && point.y == end.y {
        (
            source[projection.segment + 1..].to_vec(),
            source[..=projection.segment + 1].to_vec(),
        )
    } else {
        let mut suffix = vec![point];
        suffix.extend_from_slice(&source[projection.segment + 1..]);
        let mut prefix = source[..=projection.segment].to_vec();
        prefix.push(point);
        (suffix, prefix)
    };
    let make = |points: Vec<Point3>| {
        (points.len() >= 2).then_some(ExtrusionPath {
            polyline: Polyline3 { points },
            role: path.role,
            mm3_per_mm: path.mm3_per_mm,
            width: path.width,
            height: path.height,
        })
    };
    (make(suffix), make(prefix))
}

#[cfg(test)]
mod tests;
