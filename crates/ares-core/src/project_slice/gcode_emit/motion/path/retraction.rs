pub(super) fn uses_sloped_lift(z_hop_type: crate::ZHopType) -> bool {
    z_hop_type != crate::ZHopType::Normal
}

pub(in crate::project_slice::gcode_emit::motion) fn can_skip_retraction(
    reduce_infill_retraction: bool,
    has_sparse_infill: bool,
    previous_feature: Option<&str>,
    current_is_perimeter: bool,
    inside_internal_surface: bool,
) -> bool {
    reduce_infill_retraction
        && has_sparse_infill
        && !matches!(previous_feature, Some("Outer wall" | "Overhang wall"))
        && !current_is_perimeter
        && inside_internal_surface
}

/// The routed travel polyline in coord_t (`GCode.cpp:7421-7424`): the detour
/// vertices recovered from their mm representation, ending at the scaled
/// destination itself. Detour vertices originate as integers and round-trip
/// through mm, so they recover via `checked_scale_rounded` the same way
/// `polyline_length_scaled` does (`start_travel/route.rs`).
pub(in crate::project_slice::gcode_emit::motion) fn routed_travel(
    origin: crate::geometry::Point,
    route: &[super::super::arc::Point],
    destination: crate::geometry::Point,
    scale: crate::geometry::CoordinateScale,
    offset: (f64, f64),
) -> crate::geometry::Polyline {
    let mut points = Vec::with_capacity(route.len() + 1);
    points.push(origin);
    points.extend(route[..route.len() - 1].iter().map(|point| {
        crate::geometry::Point::new(
            scale
                .checked_scale_rounded(point.x - offset.0)
                .expect("avoid-crossing detour vertex stays in coord_t range"),
            scale
                .checked_scale_rounded(point.y - offset.1)
                .expect("avoid-crossing detour vertex stays in coord_t range"),
        )
    }));
    points.push(destination);
    crate::geometry::Polyline::new(points)
}

/// `GCode.cpp:7618-7632`: one support island or Tree base must contain the entire travel.
pub(in crate::project_slice::gcode_emit::motion) fn inside_support_islands(
    areas: super::super::support::LayerAreas<'_>,
    [start, destination]: [crate::geometry::Point; 2],
    route: &[super::super::arc::Point],
    geometry: super::super::LayerGeometry<'_>,
    offset: (f64, f64),
) -> bool {
    let point = |point: super::super::arc::Point| {
        crate::geometry::Point::new(
            ((point.x - offset.0) / geometry.scale.factor()).round() as i64,
            ((point.y - offset.1) / geometry.scale.factor()).round() as i64,
        )
    };
    let travel = crate::geometry::Polyline::new(
        std::iter::once(start)
            .chain(route[..route.len() - 1].iter().copied().map(point))
            .chain(std::iter::once(destination))
            .collect(),
    );
    areas
        .islands
        .iter()
        .chain(areas.tree_base.into_iter().flatten())
        .any(|island| crate::geometry::open_polyline_inside_expolygon(&travel, island).unwrap())
}
