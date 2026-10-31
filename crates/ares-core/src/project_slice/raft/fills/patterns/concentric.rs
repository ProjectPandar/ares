//! `FillConcentric::_fill_surface_single` (`FillConcentric.cpp:12-77`)
//! under SupportCommon's multiline1/dont_adjust/loop_clipping0 contract.

use crate::fill::concentric_sparse::nearest_point_index;
use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, JoinType, Point, Polyline, offset_expolygon,
    offset2_ex, union_contours,
};

pub(in crate::project_slice) fn concentric(
    expolygon: &ExPolygon,
    spacing: f64,
    density: f32,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let delta = (-0.5 * spacing / scale.factor()) as f32;
    let mut output = Vec::new();
    for component in offset_expolygon(expolygon, delta, JoinType::Miter, 3.0)? {
        // Native truncates scale_(spacing) to coord_t before the float
        // density division, and halves that integer using integer division.
        let min_spacing = (spacing / scale.factor()) as i64;
        let distance = (min_spacing as f32 / density) as i64;
        let half_spacing = min_spacing / 2;
        let mut last = vec![component];
        let mut loops = last
            .iter()
            .flat_map(|region| std::iter::once(region.contour()).chain(region.holes()))
            .cloned()
            .collect::<Vec<_>>();
        while !last.is_empty() {
            last = offset2_ex(
                &last,
                -(distance + half_spacing) as f32,
                half_spacing as f32,
                JoinType::Miter,
                3.0,
            )?;
            loops.extend(
                last.iter()
                    .flat_map(|region| std::iter::once(region.contour()).chain(region.holes()))
                    .cloned(),
            );
        }
        let mut last_position = Point::new(0, 0);
        for points in union_contours(&loops, scale)? {
            let start = nearest_point_index(last_position, &points);
            let mut path = Vec::with_capacity(points.len() + 1);
            path.extend_from_slice(&points[start..]);
            path.extend_from_slice(&points[..=start]);
            last_position = points[start];
            output.push(Polyline::new(path));
        }
    }
    Ok(output)
}
