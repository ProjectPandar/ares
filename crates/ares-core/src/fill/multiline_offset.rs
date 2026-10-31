use crate::geometry::{
    ClipperError, CoordinateScale, EndType, JoinType, Polygon, Polyline, offset_lines,
};

/// `FillBase.cpp:2947–3004`: expand centerlines using miter joins and round caps.
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
    // FillBase.cpp:2739-2749: spacing is float; i * spacing stays f32,
    // while 0.5 * spacing and the even-offset sum are double.
    let spacing = spacing as f32;
    let rings = multiline / 2;
    for index in 0..rings {
        let offset = if multiline % 2 == 0 {
            0.5 * f64::from(spacing) + f64::from(index as f32 * spacing)
        } else {
            f64::from((index + 1) as f32 * spacing)
        };
        let delta = (offset / scale.factor()) as f32;
        for polygon in offset_lines(
            &source,
            delta,
            JoinType::Miter,
            3.0,
            (EndType::Round, 0.002 * f64::from(delta)),
        )? {
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
