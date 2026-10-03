//! `TreeSupport.cpp::_make_loops` and `make_perimeter_and_inner_brim`.

use crate::{
    ExtrusionRole,
    geometry::{ClipperError, CoordinateScale, ExPolygon, JoinType, offset_expolygon, offset2_ex},
    project_slice::{perimeters::types::Flow, support_material::SupportExtrusionEntity},
};
use std::collections::VecDeque;

pub(super) fn make_loops(
    areas: Vec<ExPolygon>,
    wall_count: usize,
    flow: Flow,
    role: ExtrusionRole,
    scale: CoordinateScale,
) -> Result<Vec<SupportExtrusionEntity>, ClipperError> {
    let mut work = areas
        .into_iter()
        .map(|area| (area, 0))
        .collect::<VecDeque<_>>();
    let mut loops = Vec::new();
    let spacing = flow.scaled_spacing(scale) as f32;
    while let Some((area, depth)) = work.pop_front() {
        if depth + 1 < wall_count {
            for child in offset2_ex(
                std::slice::from_ref(&area),
                (-1.4 * f64::from(spacing)) as f32,
                (0.4 * f64::from(spacing)) as f32,
                JoinType::Miter,
                3.0,
            )? {
                work.push_front((child, depth + 1));
            }
        }
        let (contour, holes) = area.into_parts();
        loops.push(contour);
        loops.extend(holes);
    }
    let mut entities = Vec::new();
    SupportExtrusionEntity::append_loops(
        &mut entities,
        loops,
        role,
        (f64::from(flow.mm3_per_mm as f32), flow.width, flow.height),
    );
    Ok(entities)
}

pub(super) fn perimeter_and_inner_brim(
    area: &ExPolygon,
    wall_count: usize,
    flow: Flow,
    role: ExtrusionRole,
    scale: CoordinateScale,
) -> Result<Vec<SupportExtrusionEntity>, ClipperError> {
    let areas = offset_expolygon(
        area,
        -0.5 * flow.scaled_spacing(scale) as f32,
        JoinType::Square,
        3.0,
    )?;
    make_loops(areas, wall_count, flow, role, scale)
}
