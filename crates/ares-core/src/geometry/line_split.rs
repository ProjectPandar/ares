// `Algorithm/LineSplit.hpp::split_line` / `LineSplit.cpp:64-314::do_split_line`:
// closed paths duplicate then remove the first point; source indices preserve order.
use super::{
    ClipperError, CoordinateScale, ExPolygon, Line, LineDistanceTree, Point,
    clipper::{ClipOperation, Clipper, ClipperOptions, PathRole, z::KernelPoint},
};

const CLIP_INDEX: i64 = i64::MAX;

pub(crate) struct SplitJunction {
    pub(crate) point: Point,
    pub(crate) clipped: bool,
    source: i64,
}
impl SplitJunction {
    pub(crate) fn source_index(&self) -> usize {
        source_index(self.source)
    }
    pub(crate) fn is_source(&self) -> bool {
        self.source >= 0
    }
}
fn source_index(z: i64) -> usize {
    if z < 0 { (-z - 1) as usize } else { z as usize }
}
fn distance_squared(a: Point, b: Point) -> i64 {
    let (dx, dy) = (a.x() - b.x(), a.y() - b.y());
    dx * dx + dy * dy
}

pub(crate) fn split_line(
    points: &[Point],
    mask: &[ExPolygon],
    closed: bool,
    scale: CoordinateScale,
) -> Result<Vec<SplitJunction>, ClipperError> {
    let path: Vec<_> = points
        .iter()
        .chain(closed.then(|| &points[0]))
        .enumerate()
        .map(|(index, p)| KernelPoint::new(p.x(), p.y(), index as i64))
        .collect();
    let mut clipper = Clipper::new(ClipperOptions {
        preserve_collinear: true,
        ..ClipperOptions::default()
    });
    for polygon in mask {
        for contour in std::iter::once(polygon.contour()).chain(polygon.holes()) {
            let clip: Vec<_> = contour
                .points()
                .iter()
                .map(|p| KernelPoint::new(p.x(), p.y(), CLIP_INDEX))
                .collect();
            clipper.add_z_closed_path(&clip, PathRole::Clip)?;
        }
    }
    clipper.add_z_open_path(&path, PathRole::Subject)?;
    let mut segments = clipper.execute_z_source_paths(ClipOperation::Intersection);
    if segments.is_empty() {
        return Ok(Vec::new());
    }
    let lines: Vec<_> = path
        .windows(2)
        .map(|p| Line::new(p[0].xy, p[1].xy))
        .collect();
    let mut tree = None;
    let mut chain = vec![Vec::new(); path.len()];
    for (segment_index, segment) in segments.iter_mut().enumerate() {
        for point in segment.iter_mut() {
            if point.z == CLIP_INDEX {
                let tree = tree.get_or_insert_with(|| LineDistanceTree::new(&lines));
                resolve_clip(point, &path, tree, 1e-4 / scale.factor());
            }
        }
        super::fixed_gcc_sort_by(segment, |a, b| less(*a, *b, &path));
        let front = segment[0];
        let node = &mut chain[source_index(front.z)];
        if front.z >= 0 {
            node.insert(0, segment_index);
        } else {
            // Sort insertion uses the front point's distance on its source edge.
            node.push(segment_index);
        }
    }
    // Finish ordered insertion after all segment fronts are available.
    for node in &mut chain {
        node.sort_by(|&a, &b| {
            let (a, b) = (segments[a][0], segments[b][0]);
            if a.z >= 0 || b.z >= 0 {
                return (a.z < 0).cmp(&(b.z < 0));
            }
            let source = path[source_index(a.z)].xy;
            distance_squared(a.xy, source).cmp(&distance_squared(b.xy, source))
        });
    }
    let mut result = reconstruct(&path, &segments, &chain);
    if closed {
        result.pop();
    }
    Ok(result)
}

fn less(a: KernelPoint, b: KernelPoint, path: &[KernelPoint]) -> bool {
    if a.z < 0 && b.z < 0 && a.z == b.z {
        let source = path[source_index(a.z)].xy;
        return distance_squared(a.xy, source) < distance_squared(b.xy, source);
    }
    let (a_index, b_index) = (source_index(a.z), source_index(b.z));
    if a_index == b_index {
        a.z >= 0
    } else {
        a_index < b_index
    }
}

fn reconstruct(
    path: &[KernelPoint],
    segments: &[Vec<KernelPoint>],
    chain: &[Vec<usize>],
) -> Vec<SplitJunction> {
    let mut result: Vec<SplitJunction> = Vec::new();
    let mut index = 0;
    while index < chain.len() {
        let point = path[index];
        let node = &chain[index];
        if node.is_empty() {
            result.push(SplitJunction {
                point: point.xy,
                clipped: false,
                source: index as i64,
            });
            index += 1;
        } else {
            if segments[node[0]][0].z < 0
                && result
                    .last()
                    .is_none_or(|p| p.source_index() != source_index(point.z))
            {
                result.push(SplitJunction {
                    point: point.xy,
                    clipped: false,
                    source: index as i64,
                });
            }
            for &segment in node {
                result.extend(segments[segment].iter().map(|point| SplitJunction {
                    point: point.xy,
                    clipped: true,
                    source: point.z,
                }));
                result.last_mut().expect("a split has an endpoint").clipped = false;
            }
            let back = result.last().expect("a split has an endpoint").source;
            if back < 0 {
                let next = source_index(back);
                index = next + usize::from(next == index || chain[next].is_empty());
            } else {
                result.pop();
                index = back as usize;
            }
        }
    }
    result
}

fn resolve_clip(
    point: &mut KernelPoint,
    path: &[KernelPoint],
    tree: &LineDistanceTree,
    radius: f64,
) {
    let edges = tree.all_lines_in_radius(point.xy, radius);
    for &edge in &edges {
        let (a, b) = (path[edge], path[edge + 1]);
        if point.xy == a.xy {
            point.z = a.z;
            return;
        }
        if point.xy == b.xy {
            point.z = b.z;
            return;
        }
        let (dx, dy) = (b.x() - a.x(), b.y() - a.y());
        let (px, py) = (point.x() - a.x(), point.y() - a.y());
        let between = if a.x() != b.x() {
            (point.x() > a.x()) == (point.x() < b.x())
        } else {
            (point.y() > a.y()) == (point.y() < b.y())
        };
        if dx * py == dy * px && between {
            point.z = -(a.z + 1);
            return;
        }
    }
    // Upstream's explicit near-edge fallback, preserving tree traversal order.
    point.z = -(path[*edges.first().expect("clip vertex is near a source edge")].z + 1);
}
