use super::super::variable_width;
use super::{TraverseExtrusionsContext, unsupported};
use crate::{
    SliceError,
    arachne::ExtrusionLine,
    geometry::{BoundingBox, clip_clipper_polygons_with_subject_bbox, clip_open_path_widths},
    project_slice::perimeters::classic::{
        materialize::{ExtrusionPath, ExtrusionRole},
        shortest_path::chain_and_reorder_extrusion_paths,
    },
};

const SCALED_EPSILON_MM: f64 = 1e-4;

/// The overhang detection branch of `traverse_extrusions`
/// (`PerimeterGenerator.cpp:389-524`): intersect the variable-width wall
/// polyline with the nozzle-half-grown lower slices, append the outside
/// remainder as overhang perimeter, then re-chain from a start point. The
/// upstream `clip_extrusion` (`:280-357`) supplies crossing width interpolation
/// and zero-width subject projection repair. Non-positive output widths
/// violate its internal positive-width contract.
pub(super) fn append_overhang_clipped_paths(
    paths: &mut Vec<ExtrusionPath>,
    extrusion: &ExtrusionLine,
    role: ExtrusionRole,
    context: &TraverseExtrusionsContext<'_>,
    detect_steep: bool,
) -> Result<bool, SliceError> {
    let subject = extrusion
        .junctions
        .iter()
        .map(|junction| (junction.point.x(), junction.point.y(), junction.width))
        .collect::<Vec<_>>();
    let mut bounds = BoundingBox::from_polygon(&extrusion.to_polygon())
        .ok_or_else(|| unsupported("detect_overhang_wall"))?;
    bounds.offset(
        context
            .scale
            .checked_scale(SCALED_EPSILON_MM)
            .ok_or_else(|| unsupported("detect_overhang_wall"))?,
    );
    let lower = clip_clipper_polygons_with_subject_bbox(context.lower_slices_polygons, bounds);
    let clip = |operation| {
        clip_open_path_widths(&subject, &lower, operation).map_err(|_| {
            SliceError::InvalidInput(
                "Arachne overhang clipping is outside the supported Clipper range".to_owned(),
            )
        })
    };
    let is_external = extrusion.inset_index == 0;
    let inside = clip(crate::geometry::ClipOperation::Intersection)?;
    let overhang = clip(crate::geometry::ClipOperation::Difference)?;
    debug_assert!(
        inside
            .iter()
            .chain(&overhang)
            .all(|path| path.iter().all(|&(_, _, width)| width > 0))
    );
    variable_width::append_clipped_paths(
        paths,
        &inside,
        role,
        if is_external {
            context.ext_perimeter_flow
        } else {
            context.perimeter_flow
        },
        context.scale,
    )?;
    let steep = detect_steep
        && super::super::steep_overhang::detect(
            paths,
            context.overhang_reverse_threshold,
            context.lower_slices,
            context.scale,
        )?;
    variable_width::append_clipped_paths(
        paths,
        &overhang,
        ExtrusionRole::OverhangPerimeter,
        context.overhang_flow,
        context.scale,
    )?;
    if !paths.is_empty() {
        // Reapply the nearest point search for a starting point
        // (`PerimeterGenerator.cpp:437-462,499-500`).
        let start = if extrusion.is_closed {
            paths[0].polyline.points[0]
        } else {
            select_open_start(paths)
        };
        chain_and_reorder_extrusion_paths(paths, [start.x, start.y]);
    }
    Ok(steep
        || ((context.overhang_reverse && context.layer_id % 2 == 1)
            && paths
                .iter()
                .any(|path| path.role == ExtrusionRole::OverhangPerimeter)))
}

/// Open-wall endpoint preference (PerimeterGenerator.cpp:472–497).
fn select_open_start(
    paths: &[ExtrusionPath],
) -> crate::project_slice::perimeters::classic::materialize::Point3 {
    use crate::geometry::{Point, point_hash_map::PointHashMap};

    let mut start = paths[0].polyline.points[0];
    let mut occurrence = PointHashMap::new();
    for path in paths {
        let first = path.polyline.points[0];
        let last = *path.polyline.points.last().expect("a path has an endpoint");
        let overhang = path.role == ExtrusionRole::OverhangPerimeter;
        for point in [first, last] {
            let position = Point::new(point.x, point.y);
            if let Some(info) = occurrence.get_mut(position) {
                let info: &mut (usize, bool, _) = info;
                info.0 += 1;
                info.1 |= overhang;
            } else {
                occurrence.insert(position, (1, overhang, point));
            }
        }
    }
    for (_, (count, overhang, point)) in occurrence.into_ordered() {
        if count != 1 {
            continue;
        }
        start = point;
        if !overhang {
            break;
        }
    }
    start
}
