use super::seam_placement::paint::{SeamPaint, paint_candidates};
use crate::{
    SliceError,
    geometry::{CoordinateScale, Point},
    project_slice::perimeters::classic::{
        chained_loops::ExtrusionLoopRole,
        entity_collections::{ExtrusionEntity, ExtrusionEntityCollection},
        materialize::{ExtrusionPath, ExtrusionRole},
    },
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::project_slice) struct SeamCandidatePosition {
    pub(in crate::project_slice) x: f32,
    pub(in crate::project_slice) y: f32,
    pub(in crate::project_slice) z: f32,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct SeamPerimeter {
    pub(in crate::project_slice) start_index: usize,
    pub(in crate::project_slice) end_index: usize,
    pub(in crate::project_slice) flow_width: f32,
    pub(in crate::project_slice) is_hole: bool,
    pub(in crate::project_slice) contour_reliable: bool,
    pub(in crate::project_slice) exact_paint_ends: Option<ExactPaintEnds>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct ExactPaintEnds {
    pub(in crate::project_slice) enforced: bool,
    pub(in crate::project_slice) blocked: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::project_slice) enum PaintState {
    Blocked,
    #[default]
    Neutral,
    Enforced,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::project_slice) struct SeamCandidate {
    pub(in crate::project_slice) position: SeamCandidatePosition,
    pub(in crate::project_slice) perimeter_index: usize,
    pub(in crate::project_slice) local_ccw_angle: f32,
    pub(in crate::project_slice) paint: PaintState,
    pub(in crate::project_slice) central_enforcer: bool,
}

#[derive(Debug, Default, PartialEq)]
pub(in crate::project_slice) struct LayerSeamCandidates {
    pub(in crate::project_slice) perimeters: Vec<SeamPerimeter>,
    pub(in crate::project_slice) points: Vec<SeamCandidate>,
}

pub(in crate::project_slice) struct RegionPerimeters<'a> {
    pub(in crate::project_slice) collection: Option<&'a ExtrusionEntityCollection>,
    pub(in crate::project_slice) external_flow_width: f32,
}

pub(in crate::project_slice) fn generate_regions(
    regions: &[RegionPerimeters<'_>],
    z: f32,
    scale: CoordinateScale,
    angle_arm_mm: f32,
    paint: Option<&SeamPaint>,
) -> Result<LayerSeamCandidates, SliceError> {
    let polygons = extract_perimeter_polygons(regions);
    let mut output = LayerSeamCandidates::default();
    for polygon in polygons {
        process_polygon(polygon, z, scale, (angle_arm_mm, paint), &mut output)?;
    }
    Ok(output)
}

#[derive(Debug, PartialEq)]
struct SourcePerimeterPolygon {
    points: Vec<Point>,
    flow_width: f32,
    boundary_hole: Option<bool>,
}

fn extract_perimeter_polygons(regions: &[RegionPerimeters<'_>]) -> Vec<SourcePerimeterPolygon> {
    let mut polygons = Vec::new();
    for region in regions {
        if let Some(collection) = region.collection {
            extract_collection(collection, region.external_flow_width, &mut polygons);
        }
    }
    if polygons.is_empty() {
        polygons.push(SourcePerimeterPolygon {
            points: vec![Point::new(0, 0)],
            flow_width: 0.0,
            boundary_hole: None,
        });
    }
    polygons
}

fn extract_collection(
    collection: &ExtrusionEntityCollection,
    external_flow_width: f32,
    polygons: &mut Vec<SourcePerimeterPolygon>,
) {
    for entity in &collection.entities {
        // OrcaSlicer v2.5.0-beta `SeamPlacer.cpp:487-497`: closed loops keep
        // exterior identity through inset 0 even when every path is overhang.
        // MultiPaths still use their entity role (the first sub-path).
        let external = match entity {
            ExtrusionEntity::Loop(ordered) => {
                ordered.inset_idx == 0
                    || ordered
                        .extrusion_loop
                        .paths
                        .iter()
                        .any(|path| path.role == ExtrusionRole::ExternalPerimeter)
            }
            ExtrusionEntity::MultiPath(multi_path) => matches!(
                multi_path.paths.first().map(|path| path.role),
                Some(ExtrusionRole::ExternalPerimeter)
            ),
            ExtrusionEntity::Path(path) => path.role == ExtrusionRole::ExternalPerimeter,
        };
        if !external {
            continue;
        }
        let paths = match entity {
            ExtrusionEntity::Loop(ordered) => &ordered.extrusion_loop.paths,
            // `ExtrusionEntity::collect_points` (`ExtrusionEntity.hpp`) appends
            // every multi-path sub-path point.
            ExtrusionEntity::MultiPath(multi_path) => &multi_path.paths,
            ExtrusionEntity::Path(path) => std::slice::from_ref(path),
        };
        polygons.push(SourcePerimeterPolygon {
            points: collect_loop_points(paths),
            flow_width: external_flow_width,
            // SeamPlacer.cpp::extract_perimeter_polygons: loop role survives
            // printing-direction reversal; open paths have unknown boundaries.
            boundary_hole: match entity {
                ExtrusionEntity::Loop(ordered) => {
                    Some(ordered.extrusion_loop.role == ExtrusionLoopRole::Hole)
                }
                ExtrusionEntity::MultiPath(_) | ExtrusionEntity::Path(_) => None,
            },
        });
    }
    if polygons.is_empty() {
        let points = collection
            .entities
            .iter()
            .flat_map(|entity| match entity {
                ExtrusionEntity::Loop(ordered) => {
                    collect_loop_points(&ordered.extrusion_loop.paths)
                }
                ExtrusionEntity::MultiPath(multi_path) => collect_loop_points(&multi_path.paths),
                ExtrusionEntity::Path(path) => collect_loop_points(std::slice::from_ref(path)),
            })
            .collect::<Vec<_>>();
        if !points.is_empty() {
            polygons.push(SourcePerimeterPolygon {
                points,
                flow_width: external_flow_width,
                boundary_hole: None,
            });
        }
    }
}

fn collect_loop_points(paths: &[ExtrusionPath]) -> Vec<Point> {
    paths
        .iter()
        .flat_map(|path| {
            path.polyline
                .points
                .iter()
                .map(|point| Point::new(point.x, point.y))
        })
        .collect()
}

fn process_polygon(
    source: SourcePerimeterPolygon,
    z: f32,
    scale: CoordinateScale,
    (angle_arm_mm, paint): (f32, Option<&SeamPaint>),
    output: &mut LayerSeamCandidates,
) -> Result<(), SliceError> {
    if source.points.is_empty() {
        return Ok(());
    }
    let mut points = source.points;
    let was_clockwise = signed_area(&points) < 0.0;
    if was_clockwise {
        points.reverse();
    }
    let lengths = edge_lengths(&points, scale);
    let angles = vertex_angles(&points, &lengths, angle_arm_mm);
    let perimeter_index = output.perimeters.len();
    let start_index = output.points.len();
    let candidates = points
        .iter()
        .zip(angles)
        .map(|(point, angle)| SeamCandidate {
            position: SeamCandidatePosition {
                x: scale.unscale(point.x()) as f32,
                y: scale.unscale(point.y()) as f32,
                z,
            },
            perimeter_index,
            local_ccw_angle: if was_clockwise { -angle } else { angle },
            paint: PaintState::Neutral,
            central_enforcer: false,
        });
    let exact_paint_ends = if let Some(paint) = paint {
        let xy = points
            .iter()
            .map(|point| [scale.unscale(point.x()), scale.unscale(point.y())])
            .collect::<Vec<_>>();
        let (samples, exact) =
            paint_candidates(paint, candidates.collect(), &xy, source.flow_width)?;
        output.points.extend(samples);
        Some(exact)
    } else {
        output.points.extend(candidates);
        None
    };
    output.perimeters.push(SeamPerimeter {
        start_index,
        end_index: output.points.len(),
        flow_width: source.flow_width,
        // SeamPlacer.cpp:722-725: these are immutable contour inputs only.
        is_hole: source.boundary_hole.unwrap_or(was_clockwise),
        contour_reliable: source.boundary_hole.is_some(),
        exact_paint_ends,
    });
    Ok(())
}

fn edge_lengths(points: &[Point], scale: CoordinateScale) -> Vec<f32> {
    let mut lengths = points
        .windows(2)
        .map(|edge| point_distance_mm(edge[0], edge[1], scale) as f32)
        .collect::<Vec<_>>();
    lengths.push(point_distance_mm(points[0], points[points.len() - 1], scale).max(0.1) as f32);
    lengths
}

fn vertex_angles(points: &[Point], lengths: &[f32], min_arm_length: f32) -> Vec<f32> {
    let mut result = vec![0.0; points.len()];
    let mut previous = 0_usize;
    let mut current = 0;
    let mut next = 0;
    let mut distance_to_previous = 0.0;
    let mut distance_to_next = 0.0;

    while distance_to_previous < min_arm_length {
        previous = previous.checked_sub(1).unwrap_or(points.len() - 1);
        distance_to_previous += lengths[previous];
    }
    while current < points.len() {
        while distance_to_previous - lengths[previous] > min_arm_length {
            distance_to_previous -= lengths[previous];
            previous = (previous + 1) % points.len();
        }
        while distance_to_next < min_arm_length {
            distance_to_next += lengths[next];
            next = (next + 1) % points.len();
        }
        result[current] = signed_angle(
            vector(points[previous], points[current]),
            vector(points[current], points[next]),
        );
        let current_distance = lengths[current];
        current += 1;
        distance_to_previous += current_distance;
        distance_to_next -= current_distance;
    }
    result
}

fn signed_area(points: &[Point]) -> f64 {
    let mut previous = points[points.len() - 1];
    let mut doubled_negative_area = 0.0;
    for &current in points {
        doubled_negative_area +=
            (previous.x() as f64 + current.x() as f64) * (previous.y() as f64 - current.y() as f64);
        previous = current;
    }
    -doubled_negative_area * 0.5
}

fn point_distance_mm(left: Point, right: Point, scale: CoordinateScale) -> f64 {
    let left_x = scale.unscale(left.x());
    let left_y = scale.unscale(left.y());
    let right_x = scale.unscale(right.x());
    let right_y = scale.unscale(right.y());
    let dx = left_x - right_x;
    let dy = left_y - right_y;
    (dx * dx + dy * dy).sqrt()
}

fn vector(from: Point, to: Point) -> [f64; 2] {
    [(to.x() - from.x()) as f64, (to.y() - from.y()) as f64]
}

fn signed_angle(first: [f64; 2], second: [f64; 2]) -> f32 {
    let cross = first[0] * second[1] - first[1] * second[0];
    let dot = first[0] * second[0] + first[1] * second[1];
    cross.atan2(dot) as f32
}
