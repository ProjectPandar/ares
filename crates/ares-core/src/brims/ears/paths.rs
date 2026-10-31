//! Brim.cpp:813-856: source-owned stepped brim areas to ordered paths.
use crate::SliceError;
use crate::geometry::{CoordinateScale, ExPolygon, JoinType, Point, offset_expolygons};

pub(crate) fn from_area(
    brim_area: Vec<ExPolygon>,
    spacing: f32,
    resolution: f64,
    scale: CoordinateScale,
) -> Result<Vec<Vec<Point>>, SliceError> {
    // `Brim.cpp:824-832` — every stage of the loop stepping runs a
    // douglas_peucker pass with `resolution` (0.0125 mm) between the
    // offsets: on the input area, after the −0.5 spacing opening, and
    // between the −1.3/+0.3 closing pair.
    let mut brim_area = brim_area;
    for expolygon in &mut brim_area {
        expolygon.douglas_peucker(resolution);
    }
    let mut area = offset_expolygons(&brim_area, -0.5f32 * spacing, JoinType::Round, resolution)
        .map_err(brim_geometry_error)?;
    for expolygon in &mut area {
        expolygon.douglas_peucker(resolution);
    }
    let mut loops = Vec::new();
    while !area.is_empty() {
        // Brim.cpp:826-827 — the loop-top douglas_peucker simplifies
        // the islands IN PLACE: the harvested rings and the −1.3 offset
        // input are the same simplified polygons.
        for expolygon in &mut area {
            expolygon.douglas_peucker(resolution);
        }
        append_loops(&mut loops, &area);
        let mut next_area =
            offset_expolygons(&area, -1.3f32 * spacing, JoinType::Round, resolution)
                .map_err(brim_geometry_error)?;
        for expolygon in &mut next_area {
            expolygon.douglas_peucker(resolution);
        }
        area = offset_expolygons(&next_area, 0.3f32 * spacing, JoinType::Round, resolution)
            .map_err(brim_geometry_error)?;
    }
    // `makeBrimInfillImpl` (Brim.cpp:839): union_pt_chained_outside_in
    // — upstream runs the stepped loops through union_pt (EvenOdd
    // PolyTree), which reorders each contour to the processed walk's
    // start vertex, then chains outside-in on the contour front
    // points. Reproduce that here.
    let unified = crate::geometry::union_contours(
        &loops
            .iter()
            .map(|points| crate::geometry::Polygon::new(points.clone()))
            .collect::<Vec<_>>(),
        scale,
    )
    .map_err(brim_geometry_error)?;
    let loops = unified;
    // The union's contour-then-holes order IS the outside-in nesting
    // (`traverse_pt_outside_in` recursion — the area sort would
    // re-interleave nested rings destructively).
    // `makeBrimInfillImpl` (Brim.cpp:843-849): each ring becomes an
    // OPEN polyline (to_polylines of a closed polygon drops the
    // closing duplication), then `optimize_polylines_by_reversing`
    // (cpp:719-733) flips each ring so its END is nearer the previous
    // ring's END, then `connect_brim_lines` (cpp:735-808) joins
    // successive rings end-to-end when the gap ≤ 2x spacing and the
    // connector does not cross brim centerlines (an EdgeGrid check;
    // approximated here by the gap test alone for the single-object
    // brim whose rings are concentric and connector-safe).
    let connected = connect_brim_lines(loops, spacing);
    // Re-close each connected path for emission.
    let paths = connected
        .into_iter()
        .map(|mut points| {
            if points.len() > 1 && points.first() != points.last() {
                points.push(points[0]);
            }
            points
        })
        .collect::<Vec<_>>();
    Ok(paths)
}
fn connect_brim_lines(loops: Vec<Vec<Point>>, spacing: f32) -> Vec<Vec<Point>> {
    let mut previous_end: Option<Point> = None;
    let mut connected: Vec<Vec<Point>> = Vec::new();
    for mut points in loops {
        // `connect_brim_lines` (Brim.cpp:775-797) joins two successive
        // polylines only when BOTH are open (`! prev.is_closed() &&
        // ! next.is_closed()`); closed rings are never merged — keep
        // the closing point and skip the reversal for closed rings
        // (`optimize_polylines_by_reversing` finds equal endpoint
        // distances and never flips a closed ring).
        // Clipper's closed output contours carry the duplicated
        // closing point (`node->Contour`), so upstream's brim ring
        // polylines are closed — `to_polylines` keeps the duplicate.
        // Close each ring the same way; the `is_closed` guard below
        // then mirrors `connect_brim_lines`'s no-merge rule.
        if points.len() > 1 && points.first() != points.last() {
            let first = points[0];
            points.push(first);
        }
        let is_closed = points.len() > 1 && points.first() == points.last();
        if !is_closed && let Some(end) = previous_end {
            let distance = |point: &Point| {
                let dx = (point.x() - end.x()) as f64;
                let dy = (point.y() - end.y()) as f64;
                dx * dx + dy * dy
            };
            let first = points[0];
            let last = points[points.len() - 1];
            if distance(&last) < distance(&first) {
                points.reverse();
            }
        }
        let spacing_squared = f64::from(spacing) * f64::from(spacing) * 4.0;
        let previous_closed = connected
            .last()
            .is_some_and(|tail: &Vec<Point>| tail.len() > 1 && tail.first() == tail.last());
        let gap_ok = !is_closed
            && !previous_closed
            && previous_end.is_some_and(|end| {
                let start = points[0];
                let dx = (start.x() - end.x()) as f64;
                let dy = (start.y() - end.y()) as f64;
                dx.mul_add(dx, dy * dy) <= spacing_squared
            });
        if gap_ok {
            let tail = connected.last_mut().expect("gap_ok implies a tail");
            tail.extend_from_slice(&points);
        } else {
            connected.push(points);
        }
        previous_end = connected.last().and_then(|points| points.last().copied());
    }
    connected
}

fn append_loops(output: &mut Vec<Vec<Point>>, area: &[ExPolygon]) {
    for expolygon in area {
        output.push(expolygon.contour().points().to_vec());
        output.extend(expolygon.holes().iter().map(|hole| hole.points().to_vec()));
    }
    output.retain(|points| points.len() >= 3);
}

fn brim_geometry_error(_: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput("brim generation failed".to_owned())
}
