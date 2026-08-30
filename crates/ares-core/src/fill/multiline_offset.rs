use crate::geometry::{
    ClipperError, CoordinateScale, JoinType, Polygon, Polyline, offset_open_paths,
};

/// `FillBase.cpp:2712-2784`: expand each sparse centerline into the configured
/// odd/even bundle using round-ended Clipper2 offsets.
pub(crate) fn apply(
    polylines: Vec<Polyline>,
    multiline: i32,
    spacing: f64,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    if multiline <= 1 {
        return Ok(polylines);
    }
    let source = polylines
        .into_iter()
        .filter(|polyline| polyline.points().len() >= 2)
        .map(|polyline| Polygon::new(polyline.into_points()))
        .collect::<Vec<_>>();
    if source.is_empty() {
        return Ok(Vec::new());
    }

    let mut output = Vec::new();
    if multiline % 2 != 0 {
        output.extend(
            source
                .iter()
                .map(|path| Polyline::new(path.points().to_vec())),
        );
    }
    let rings = multiline / 2;
    for index in 0..rings {
        let offset = if multiline % 2 == 0 {
            0.5 * spacing + f64::from(index) * spacing
        } else {
            f64::from(index + 1) * spacing
        };
        let delta = (offset / scale.factor()) as f32;
        for polygon in offset_open_paths(&source, delta, JoinType::Round, 2.0)? {
            let mut points = polygon.into_points();
            if points.len() < 3 {
                continue;
            }
            if points.first() != points.last() {
                points.push(points[0]);
            }
            output.push(Polyline::new(points));
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests;
