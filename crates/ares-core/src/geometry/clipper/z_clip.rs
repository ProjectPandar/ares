// ClipperZUtils.cpp and SupportCommon.cpp: preserve caller metadata on C2 paths.
use super::z::{KernelPoint, clip_zpaths};
use super::{ClipOperation, ClipperError, JoinType, raw_offset_paths};
use crate::geometry::{ExPolygon, Polygon};

pub(crate) type ZWidthPath = Vec<(i64, i64, i64)>;

pub(crate) fn clip_open_path_widths(
    subject: &ZWidthPath,
    clip: &[Polygon],
    operation: ClipOperation,
) -> Result<Vec<ZWidthPath>, ClipperError> {
    let subject_path: Vec<_> = subject
        .iter()
        .map(|&(x, y, z)| KernelPoint::new(x, y, z))
        .collect();
    let clip_paths: Vec<_> = clip
        .iter()
        .map(|p| {
            p.points()
                .iter()
                .map(|&xy| KernelPoint { xy, z: 0 })
                .collect()
        })
        .collect();
    let mut paths = clip_zpaths(
        std::slice::from_ref(&subject_path),
        (true, false),
        &clip_paths,
        operation,
        |a, b, c, d, point| {
            let (mut start, mut end) = (a, b);
            if start.z <= 0 && end.z <= 0 {
                start = c;
                end = d;
            }
            let dx = (end.x - start.x) as f64;
            let dy = (end.y - start.y) as f64;
            let dz = (end.z - start.z) as f64;
            let length = dx * dx + dy * dy + dz * dz;
            let px = (point.x - start.x) as f64;
            let py = (point.y - start.y) as f64;
            let pz = (point.z - start.z) as f64;
            let distance = px * px + py * py + pz * pz;
            let t = (distance / length).sqrt();
            point.z = start.z + ((end.z - start.z) as f64 * t) as i64;
        },
    );
    for point in paths.iter_mut().flatten().filter(|point| point.z == 0) {
        if subject_path.len() <= 2 {
            continue;
        }
        let mut nearest = (&subject_path[0], &subject_path[1], subject_path[0]);
        let mut minimum = f64::MAX;
        for segment in subject_path.windows(2) {
            let (a, b) = (&segment[0], &segment[1]);
            let dx = (b.x() - a.x()) as f64;
            let dy = (b.y() - a.y()) as f64;
            let theta = ((b.x() - point.x()) as f64 * dx + (b.y() - point.y()) as f64 * dy)
                / (dx * dx + dy * dy);
            let projected = if a.xy == b.xy {
                *a
            } else if (0.0..=1.0).contains(&theta) {
                KernelPoint::new(
                    (theta * a.x() as f64 + (1.0 - theta) * b.x() as f64) as i64,
                    (theta * a.y() as f64 + (1.0 - theta) * b.y() as f64) as i64,
                    0,
                )
            } else if distance_squared(*a, *point) < distance_squared(*b, *point) {
                *a
            } else {
                *b
            };
            let distance = distance_squared(projected, *point);
            if distance < minimum {
                minimum = distance;
                nearest = (a, b, projected);
            }
        }
        let (a, b, projected) = nearest;
        let length = distance_squared(*a, *b).sqrt();
        let distance = distance_squared(projected, *a).sqrt();
        point.z = (a.z as f64 + distance / length * (b.z - a.z) as f64) as i64;
    }
    Ok(paths
        .into_iter()
        .map(|path| {
            path.into_iter()
                .map(|point| (point.x(), point.y(), point.z))
                .collect::<ZWidthPath>()
        })
        .collect())
}

fn distance_squared(a: KernelPoint, b: KernelPoint) -> f64 {
    let dx = (a.x() - b.x()) as f64;
    let dy = (a.y() - b.y()) as f64;
    dx * dx + dy * dy
}

pub(crate) type ZTaggedPath = Vec<(i64, i64, i64)>;

// SupportCommon.cpp:shrink_expolygon_with_contour_idx/offset_ccw.
pub(crate) fn shrink_expolygon_with_contour_idx(
    expolygon: &ExPolygon,
    delta: f32,
    kind: JoinType,
    limit: f64,
) -> Result<Vec<ZTaggedPath>, ClipperError> {
    let offset = |polygon: &Polygon, delta, tag| -> Result<Vec<Vec<KernelPoint>>, ClipperError> {
        let mut polygon = polygon.clone();
        if !polygon.is_counter_clockwise() {
            polygon.reverse();
        }
        Ok(raw_offset_paths(&[polygon], delta, kind, limit)?
            .into_iter()
            .map(|p| {
                p.into_points()
                    .into_iter()
                    .map(|xy| KernelPoint { xy, z: tag })
                    .collect()
            })
            .collect())
    };
    let contours = offset(expolygon.contour(), -delta, 0)?;
    if contours.is_empty() {
        return Ok(Vec::new());
    }
    let mut holes = Vec::new();
    for (i, hole) in expolygon.holes().iter().enumerate() {
        holes.extend(offset(hole, delta, (i + 1) as i64)?);
    }
    let paths = if holes.is_empty() {
        contours
    } else {
        clip_zpaths(
            &contours,
            (false, false),
            &holes,
            ClipOperation::Difference,
            |_, _, _, _, point| point.z = -1,
        )
    };
    Ok(paths
        .into_iter()
        .map(|path| path.into_iter().map(|p| (p.x(), p.y(), p.z)).collect())
        .collect())
}
