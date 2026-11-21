//! Classic and Arachne spiral chaining (FillSpiralInset.cpp).
mod chain;
mod walls;

use crate::{
    ProcessSurfaceFillOrder,
    fill::line::adjust_solid_spacing,
    geometry::{
        ClipperError, CoordinateScale, ExPolygon, JoinType, Polygon, Polyline, offset_expolygon,
        offset2_ex, union_contours,
    },
};

pub(crate) use walls::spiral_walls;

pub(crate) fn classic(
    domain: ExPolygon,
    params: &super::FillParams,
    spacing: &mut f64,
    order: ProcessSurfaceFillOrder,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let min_spacing = scale
        .checked_scale(*spacing)
        .filter(|s| *s > 0)
        .ok_or(ClipperError::CoordinateOutOfRange)?;
    let mut distance = (min_spacing as f32 / params.density) as i64;
    if params.density > 0.9999 && !params.dont_adjust {
        let points = domain.contour().points();
        let min = points.iter().map(|p| p.x()).min().unwrap();
        let max = points.iter().map(|p| p.x()).max().unwrap();
        distance = adjust_solid_spacing(max - min, distance);
        *spacing = scale.unscale(distance);
    }
    let half_spacing = min_spacing / 2;
    let inset = distance
        .checked_add(half_spacing)
        .ok_or(ClipperError::CoordinateOutOfRange)?;
    let mut last = vec![domain];
    let mut loops = Vec::new();
    while !last.is_empty() {
        loops.extend(
            last.iter()
                .flat_map(|p| std::iter::once(p.contour()).chain(p.holes()))
                .cloned(),
        );
        last = offset2_ex(
            &last,
            -(inset as f32),
            half_spacing as f32,
            JoinType::Miter,
            3.0,
        )?;
    }
    let loops = union_contours(&loops, scale)?
        .into_iter()
        .map(Polygon::new)
        .collect::<Vec<_>>();
    // Native passes the moved-from original ExPolygon to the classic chain.
    chain::generate(&loops, &loops, distance, order, None)
}

pub(crate) fn support(
    domain: &ExPolygon,
    spacing: f64,
    density: f32,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let mut output = Vec::new();
    for domain in offset_expolygon(
        domain,
        scale.scaled_delta(-0.5 * spacing) as f32,
        JoinType::Miter,
        3.0,
    )? {
        output.extend(classic(
            domain,
            &super::FillParams {
                density,
                multiline: 1,
                dont_adjust: true,
            },
            &mut { spacing },
            ProcessSurfaceFillOrder::Default,
            scale,
        )?);
    }
    Ok(output)
}
