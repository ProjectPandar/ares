mod aabb;
mod splits;

use self::aabb::BoundaryAabb;
pub(super) use self::aabb::sample_in_expolygons;
#[cfg(test)]
pub(in crate::geometry) use self::aabb::{bbox_contains_for_test, sample_for_test};
#[cfg(test)]
pub(in crate::geometry) use self::aabb::{
    centroid_for_test, longest_axis_for_test, partition_for_test,
};
#[cfg(test)]
pub(in crate::geometry) use self::splits::{
    merge_path_for_test, reconcile_for_test, sort_seeds_for_test, split_registry_for_test,
};
use self::splits::{merge_splits, sort_seeds, split_registry};
use super::WaveSeed;
use crate::geometry::clipper::z::{KernelPoint, ZPath};
use crate::geometry::clipper::{
    ClipOperation, Clipper, ClipperError, ClipperOffset, ClipperOptions, FillRule, JoinType,
    PathRole,
};
use crate::geometry::{CoordinateScale, ExPolygon, Point, Polygon};

pub(crate) fn wave_seeds(
    src: &[ExPolygon],
    boundary: &[ExPolygon],
    tiny_expansion: f32,
    sorted: bool,
    scale: CoordinateScale,
) -> Result<Vec<WaveSeed>, ClipperError> {
    assert!(tiny_expansion > 0.0);
    if src.is_empty() || boundary.is_empty() {
        return Ok(Vec::new());
    }

    let boundary_begin = 1_i64;
    let mut boundary_end = boundary_begin;
    let mut clipper = Clipper::new(ClipperOptions::default());
    for expolygon in boundary {
        add_boundary(&mut clipper, expolygon, boundary_end)?;
        boundary_end += 1;
    }

    let mut src_end = boundary_end;
    let source_paths = expanded_source_paths(src, tiny_expansion, &mut src_end)?;
    for path in &source_paths {
        clipper.add_z_open_path(path, PathRole::Subject)?;
    }
    let mut split_records = split_registry(&source_paths);
    let (mut segments, intersections) = clipper.execute_z_paths(
        ClipOperation::Intersection,
        FillRule::NonZero,
        FillRule::NonZero,
    );
    merge_splits(&mut segments, &mut split_records);

    let mut tree = None;
    let mut output = Vec::with_capacity(segments.len());
    let recovery = RecoveryContext {
        intersections: &intersections,
        boundaries: boundary,
        boundary_begin,
        boundary_end,
        src_end,
        scale,
    };
    for path in segments {
        recover_path(path, recovery, &mut tree, &mut output);
    }
    if sorted {
        sort_seeds(&mut output);
    }
    Ok(output)
}

fn add_boundary(clipper: &mut Clipper, expolygon: &ExPolygon, z: i64) -> Result<(), ClipperError> {
    for polygon in std::iter::once(expolygon.contour()).chain(expolygon.holes()) {
        let path = polygon
            .points()
            .iter()
            .map(|point| KernelPoint { xy: *point, z })
            .collect::<ZPath>();
        clipper.add_z_closed_path(&path, PathRole::Clip)?;
    }
    Ok(())
}

fn expanded_source_paths(
    src: &[ExPolygon],
    expansion: f32,
    src_end: &mut i64,
) -> Result<Vec<ZPath>, ClipperError> {
    let mut output = Vec::new();
    let mut offsetter = ClipperOffset::default();
    offsetter.set_shortest_edge_length(f64::from(expansion) * 0.005_f64);
    for expolygon in src {
        for (index, polygon) in std::iter::once(expolygon.contour())
            .chain(expolygon.holes())
            .enumerate()
        {
            offsetter.clear();
            offsetter.add_closed_path(polygon, JoinType::Square);
            let delta = if index == 0 {
                f64::from(expansion)
            } else {
                -f64::from(expansion)
            };
            for polygon in offsetter.execute_paths(delta)? {
                let mut path = polygon
                    .into_points()
                    .into_iter()
                    .map(|xy| KernelPoint { xy, z: *src_end })
                    .collect::<ZPath>();
                let first = path[0];
                path.push(first);
                output.push(path);
            }
        }
        *src_end += 1;
    }
    Ok(output)
}

#[derive(Clone, Copy)]
struct RecoveryContext<'a> {
    intersections: &'a [(i64, i64)],
    boundaries: &'a [ExPolygon],
    boundary_begin: i64,
    boundary_end: i64,
    src_end: i64,
    scale: CoordinateScale,
}

fn recover_path<'a>(
    path: ZPath,
    context: RecoveryContext<'a>,
    tree: &mut Option<BoundaryAabb<'a>>,
    output: &mut Vec<WaveSeed>,
) {
    debug_assert!(path.len() >= 2);
    let front = path[0];
    let back = *path.last().unwrap();
    assert_source_topology(front, back, context.boundary_end, context.src_end);
    recover_path_after_topology(path, context, tree, output, (front, back));
}

fn assert_source_topology(front: KernelPoint, back: KernelPoint, boundary_end: i64, src_end: i64) {
    debug_assert!(
        (front == back && (boundary_end..src_end).contains(&front.z)) || front.z < 0 || back.z < 0
    );
}

fn recover_path_after_topology<'a>(
    path: ZPath,
    context: RecoveryContext<'a>,
    tree: &mut Option<BoundaryAabb<'a>>,
    output: &mut Vec<WaveSeed>,
    endpoints: (KernelPoint, KernelPoint),
) {
    let (mut front, mut back) = endpoints;
    let RecoveryContext {
        boundary_begin,
        boundary_end,
        src_end,
        ..
    } = context;

    if front != back && front.z >= 0 && back.z >= 0 {
        let mut source = None;
        let mut boundary = None;
        for point in &path {
            if (boundary_end..src_end).contains(&point.z) && source.is_none() {
                source = Some(point.z);
            } else if (boundary_begin..boundary_end).contains(&point.z) && boundary.is_none() {
                boundary = Some(point.z);
            }
            if source.is_some() && boundary.is_some() {
                break;
            }
        }
        let Some(source) = source else { return };
        let boundary = boundary.map(|z| (z - 1) as u32).or_else(|| {
            sample_boundary(tree, context.boundaries, front.xy, context.scale)
                .map(|index| index as u32)
        });
        if let Some(boundary) = boundary {
            output.push(seed(path, (source - boundary_end) as u32, boundary));
        }
        return;
    }

    if front == back && front.z < boundary_end {
        for point in &path {
            if point.z >= boundary_end {
                front = *point;
                back = *point;
            }
        }
    }

    let valid = |pair: (i64, i64)| {
        (boundary_begin..boundary_end).contains(&pair.0)
            && (boundary_end..src_end).contains(&pair.1)
    };
    let mut pair = None;
    if front.z < 0 {
        let candidate = context.intersections[(-front.z - 1) as usize];
        debug_assert!(valid(candidate));
        if valid(candidate) {
            pair = Some(candidate);
        }
    }
    if pair.is_none() && back.z < 0 {
        let candidate = context.intersections[(-back.z - 1) as usize];
        debug_assert!(valid(candidate));
        if valid(candidate) {
            pair = Some(candidate);
        }
    }
    if let Some((boundary, source)) = pair {
        output.push(seed(
            path,
            (source - boundary_end) as u32,
            (boundary - 1) as u32,
        ));
        return;
    }

    debug_assert!(front == back);
    debug_assert!((boundary_end..src_end).contains(&front.z));
    let boundary = sample_boundary(tree, context.boundaries, front.xy, context.scale);
    debug_assert!(boundary.is_some());
    if let Some(boundary) = boundary {
        output.push(seed(path, (front.z - boundary_end) as u32, boundary as u32));
    }
}

fn sample_boundary<'a>(
    tree: &mut Option<BoundaryAabb<'a>>,
    boundaries: &'a [ExPolygon],
    point: Point,
    scale: CoordinateScale,
) -> Option<usize> {
    tree.get_or_insert_with(|| BoundaryAabb::build(boundaries, scale))
        .sample(point)
}

fn seed(path: ZPath, src: u32, boundary: u32) -> WaveSeed {
    WaveSeed {
        src,
        boundary,
        path: Polygon::new(path.into_iter().map(|point| point.xy).collect()),
    }
}

#[cfg(test)]
pub(in crate::geometry) fn expanded_source_paths_for_test(
    src: &[ExPolygon],
    expansion: f32,
    first_id: i64,
) -> Result<(Vec<ZPath>, i64), ClipperError> {
    let mut end = first_id;
    let paths = expanded_source_paths(src, expansion, &mut end)?;
    Ok((paths, end))
}

#[cfg(test)]
pub(in crate::geometry) fn recover_path_for_test(
    path: ZPath,
    intersections: &[(i64, i64)],
    boundaries: &[ExPolygon],
    ids: (i64, i64, i64),
    scale: CoordinateScale,
) -> (Vec<WaveSeed>, bool) {
    let mut tree = None;
    let mut output = Vec::new();
    let context = RecoveryContext {
        intersections,
        boundaries,
        boundary_begin: ids.0,
        boundary_end: ids.1,
        src_end: ids.2,
        scale,
    };
    let front = path[0];
    let back = *path.last().unwrap();
    recover_path_after_topology(path, context, &mut tree, &mut output, (front, back));
    (output, tree.is_some())
}

#[cfg(all(test, debug_assertions))]
pub(in crate::geometry) fn assert_source_topology_for_test(path: &ZPath, ids: (i64, i64)) {
    assert_source_topology(path[0], *path.last().unwrap(), ids.0, ids.1);
}
