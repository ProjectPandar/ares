//! The materialized flow for fill entities (role/bridge spacing rules).

use crate::ExtrusionRole;
use crate::project_slice::group_fills::SurfaceFillParams;
use crate::project_slice::perimeters::flow;
use crate::project_slice::perimeters::types::Flow;

pub(in crate::project_slice) fn materialized_flow(
    params: crate::project_slice::group_fills::SurfaceFillParams,
    spacing: f32,
) -> crate::project_slice::perimeters::types::Flow {
    let mut flow = if params.extrusion_role == ExtrusionRole::InternalInfill && !params.bridge {
        params.flow
    } else {
        crate::project_slice::perimeters::flow::with_spacing(params.flow, spacing)
    };
    flow.mm3_per_mm *= params.flow_ratio;
    flow
}

/// The print-object center used to anchor infill grids
/// (`Fill::_infill_direction`, `FillBase.cpp:288-290`).
pub(in crate::project_slice) fn object_center(
    object_slices: &[Vec<crate::geometry::ExPolygon>],
) -> crate::geometry::Point {
    let mut minimum_x = i64::MAX;
    let mut minimum_y = i64::MAX;
    let mut maximum_x = i64::MIN;
    let mut maximum_y = i64::MIN;
    for layer in object_slices {
        for expolygon in layer {
            for point in expolygon.contour().points() {
                minimum_x = minimum_x.min(point.x());
                minimum_y = minimum_y.min(point.y());
                maximum_x = maximum_x.max(point.x());
                maximum_y = maximum_y.max(point.y());
            }
        }
    }
    if minimum_x > maximum_x {
        return crate::geometry::Point::new(0, 0);
    }
    crate::geometry::Point::new(
        minimum_x + (maximum_x - minimum_x) / 2,
        minimum_y + (maximum_y - minimum_y) / 2,
    )
}
