//! `TreeSupport3D.cpp:749-847::safe_union/safe_offset_inc`.

use crate::geometry::{
    BoundingBox, ClipperError, Coord, CoordinateScale, JoinType, Point, Polygon,
    clip_clipper_polygons_with_subject_bbox, difference_polygons_paths, offset_open_paths,
    offset_paths, simplification::polygons_simplify, union_polygons_paths,
    union_polygons_paths_pair,
};

pub(super) struct Expansion {
    pub(super) distance: Coord,
    pub(super) safe_step: Coord,
    pub(super) unchecked_last: Coord,
    pub(super) minimum_offsets: usize,
}

pub(super) fn safe_union(
    first: &[Polygon],
    second: &[Polygon],
    scale: CoordinateScale,
) -> Result<Vec<Polygon>, ClipperError> {
    if first.is_empty() && second.is_empty() {
        return Ok(Vec::new());
    }
    let result = union_polygons_paths_pair(first, second)?;
    if !result.is_empty() {
        return Ok(result);
    }
    let open_offset = |polygons: &[Polygon]| {
        let paths = polygons
            .iter()
            .map(|polygon| Polygon::new(polygon.split_at_first_point().into_points()))
            .collect::<Vec<_>>();
        offset_open_paths(
            &paths,
            scale.scaled_delta(0.002) as f32,
            JoinType::Miter,
            1.2,
        )
    };
    union_polygons_paths_pair(&open_offset(first)?, &open_offset(second)?)
}

pub(super) fn safe_offset_inc(
    area: &[Polygon],
    collision: &[Polygon],
    expansion: Expansion,
    scale: CoordinateScale,
) -> Result<Vec<Polygon>, ClipperError> {
    let Expansion {
        distance,
        safe_step,
        unchecked_last,
        minimum_offsets,
    } = expansion;
    let mut final_difference = unchecked_last == 0;
    let mut result = safe_union(area, &[], scale)?;
    let mut collision_buffer = Vec::new();
    let epsilon = scale.scaled_delta(1e-4);
    let padding = (distance.max(0) as f64 + epsilon) as Coord;
    if distance == 0 || safe_step < 0 || unchecked_last < 0 {
        return if final_difference {
            difference_polygons_paths(
                &result,
                clipped_collision(&mut collision_buffer, collision, &result, padding),
            )
        } else {
            union_polygons_paths(&result)
        };
    }
    let mut step = safe_step;
    // The source narrows its Coord quotient and size_t minimum to int.
    let mut steps = if distance > unchecked_last {
        ((distance - unchecked_last) / step) as i32
    } else {
        0
    };
    if distance - i64::from(steps) * step > unchecked_last {
        if i64::from(steps + 1) * step <= distance {
            steps += 1;
        } else {
            final_difference = true;
        }
    }
    if steps + i32::from(distance < unchecked_last || distance % step != 0) < minimum_offsets as i32
        && minimum_offsets > 1
    {
        step = distance / minimum_offsets as Coord;
        if step >= safe_step {
            step = safe_step;
            steps = minimum_offsets as i32;
        } else {
            steps = (distance / step) as i32;
        }
    }
    let arc_tolerance = f64::from(scale.scaled_delta(0.01) as f32);
    let simplify_tolerance = scale.scaled_delta(0.015);
    for index in 0..steps {
        let grown = offset_paths(&result, step as f32, JoinType::Round, arc_tolerance)?;
        result = difference_polygons_paths(
            &grown,
            clipped_collision(&mut collision_buffer, collision, &result, padding),
        )?;
        if index % 10 == 7 {
            result = polygons_simplify(result, simplify_tolerance)?;
        }
    }
    let remainder = distance - i64::from(steps) * step;
    if f64::from(remainder as f32) > epsilon {
        result = offset_paths(&result, remainder as f32, JoinType::Round, arc_tolerance)?;
    }
    result = polygons_simplify(result, simplify_tolerance)?;
    if final_difference {
        result = difference_polygons_paths(
            &result,
            clipped_collision(&mut collision_buffer, collision, &result, padding),
        )?;
    }
    union_polygons_paths(&result)
}

fn clipped_collision<'a>(
    buffer: &'a mut Vec<Polygon>,
    collision: &[Polygon],
    area: &[Polygon],
    padding: Coord,
) -> &'a [Polygon] {
    if buffer.is_empty() && !collision.is_empty() {
        // get_extents(empty) retains BoundingBoxBase's zero coordinates.
        let bounds = BoundingBox::from_polygons(area)
            .unwrap_or(BoundingBox::new(Point::new(0, 0), Point::new(0, 0)));
        *buffer = clip_clipper_polygons_with_subject_bbox(collision, bounds.inflated(padding));
    }
    buffer
}
