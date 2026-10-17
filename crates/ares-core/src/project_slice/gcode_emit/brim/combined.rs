//! Brim.cpp:958-1018: global area merge, cleanup, infill and carrier.
use super::{BrimPlan, brim_geometry_error};
use crate::{
    ProcessDraftShield, ProcessPrintSequence, ProcessSkirtType, SliceError,
    geometry::{ExPolygon, JoinType, Point, offset2_ex, union_expolygons},
    project_slice::{
        gcode_emit::{footprint, layers::object_order},
        perimeters::classic::traversal::PreparedPostClassicTraversal,
    },
};

pub(super) fn enabled(traversal: &PreparedPostClassicTraversal) -> bool {
    let print = &traversal.resolved.views.full.process.print;
    let infinite = print.draft_shield == ProcessDraftShield::Enabled && print.skirt_loops.0 > 0;
    print.combine_brims.0
        && !(print.skirt_type == ProcessSkirtType::PerObject
            && (print.skirt_height.0 > 0 || infinite))
        && print.print_sequence != ProcessPrintSequence::ByObject
}

pub(in crate::project_slice::gcode_emit) fn combine(
    traversal: &PreparedPostClassicTraversal,
    plans: &mut [Option<BrimPlan>],
) -> Result<(), SliceError> {
    if !enabled(traversal) {
        return Ok(());
    }
    let scale = traversal.scale;
    let mut area = Vec::new();
    let mut owners = Vec::new();
    let mut centers = Vec::new();
    for (index, plan) in plans.iter_mut().enumerate() {
        let (resolved, group) = traversal.print_object_config(index);
        let object = &traversal.project.objects()[resolved.source_object_index];
        let center = crate::project_slice::centering::center(object, group, scale)
            .expect("sliced object retains its raw center");
        for &member in &group.instance_indices {
            let shift = footprint::instance_shift(&object.instances()[member], center, scale);
            owners.push(index);
            centers.push((shift.x(), shift.y()));
            if let Some(plan) = plan {
                area.extend(
                    plan.area
                        .iter()
                        .cloned()
                        .map(|polygon| translate(polygon, shift)),
                );
            }
        }
    }
    if area.is_empty() {
        return Ok(());
    }
    // Print.cpp:2520-2530 builds objPrintVec from the unseeded instance chain,
    // not the reversed, wipe-tower-seeded per-tool emission schedule.
    let carrier = object_order::chain_without_start(&centers)
        .into_iter()
        .map(|instance| owners[instance])
        .find(|&owner| plans[owner].is_some())
        .expect("a nonempty brim area has a contributing owner");
    let area = union_expolygons(&area, &[]).map_err(brim_geometry_error)?;
    let resolution = scale.scaled_delta(traversal.resolved.views.full.process.print.resolution.0);
    let delta = (resolution as f32).max(scale.scaled_delta(0.0001) as f32);
    let area = offset2_ex(&area, delta, -delta, JoinType::Round, resolution)
        .map_err(brim_geometry_error)?;
    let plan = plans[carrier]
        .as_mut()
        .expect("carrier contributed brim area");
    let spacing = super::scaled_f32(scale, f64::from(plan.spacing))?;
    plan.paths = crate::brims::ears::paths::from_area(area, spacing, scale.scaled_delta(0.0125))?;
    plan.global = true;
    Ok(())
}

fn translate(polygon: ExPolygon, shift: Point) -> ExPolygon {
    let (contour, holes) = polygon.into_parts();
    let translate_polygon = |polygon: crate::geometry::Polygon| {
        crate::geometry::Polygon::new(
            polygon
                .points()
                .iter()
                .map(|point| Point::new(point.x() + shift.x(), point.y() + shift.y()))
                .collect(),
        )
    };
    ExPolygon::new(
        translate_polygon(contour),
        holes.into_iter().map(translate_polygon).collect(),
    )
}
