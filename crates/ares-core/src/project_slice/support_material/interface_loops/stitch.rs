//! `LoopInterfaceProcessor::generate` split-point stitching after clipping.

use crate::geometry::{
    ClipperError, CoordinateScale, JoinType, Polygon, Polyline, intersection_open_polylines,
    offset_paths,
};
use std::collections::HashMap;

pub(super) fn clip_and_stitch(
    loops: &[Polygon],
    overhangs: &[Polygon],
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let mut split_points = HashMap::new();
    let input = loops
        .iter()
        .map(|polygon| {
            let point = polygon.points()[0];
            split_points.insert((point.x(), point.y()), -1_i32);
            polygon.split_at_first_point()
        })
        .collect::<Vec<_>>();
    let margin = offset_paths(
        overhangs,
        (1.5 / scale.factor()) as f32,
        JoinType::Miter,
        3.0,
    )?;
    let mut lines = intersection_open_polylines(&input, &margin)?;
    for index in 0..lines.len() {
        let line = &lines[index];
        let first = line.points()[0];
        let last = *line.points().last().unwrap();
        let (point, reverse) = if split_points.contains_key(&(first.x(), first.y())) {
            (first, false)
        } else {
            (last, true)
        };
        let Some(previous) = split_points.get_mut(&(point.x(), point.y())) else {
            continue;
        };
        if *previous < 0 {
            *previous = index as i32;
            continue;
        }
        let (before, rest) = lines.split_at_mut(index);
        let destination = &mut before[*previous as usize];
        let source = &mut rest[0];
        if destination.points()[0] == point {
            destination.reverse();
        }
        if reverse {
            source.reverse();
        }
        let points = std::mem::take(source.points_mut());
        let skip = usize::from(destination.points().last() == points.first());
        destination.append_points(points.into_iter().skip(skip));
        *previous = -2;
    }
    lines.retain(Polyline::is_valid);
    Ok(lines)
}
