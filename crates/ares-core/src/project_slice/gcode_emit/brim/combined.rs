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
    plan.paths =
        crate::brims::ears::paths::from_area(area, spacing, scale.scaled_delta(0.0125), scale)?;
    plan.global = true;
    Ok(())
}

pub(in crate::project_slice::gcode_emit) enum GroupBrim {
    Single(usize),
    Combined { owners: Vec<usize>, plan: BrimPlan },
}

impl GroupBrim {
    pub(in crate::project_slice::gcode_emit) fn owners(&self) -> &[usize] {
        match self {
            Self::Single(owner) => std::slice::from_ref(owner),
            Self::Combined { owners, .. } => owners,
        }
    }

    pub(in crate::project_slice::gcode_emit) fn plan<'a>(
        &'a self,
        plans: &'a [Option<BrimPlan>],
    ) -> &'a BrimPlan {
        match self {
            Self::Single(owner) => plans[*owner].as_ref().unwrap(),
            Self::Combined { plan, .. } => plan,
        }
    }
}

pub(in crate::project_slice::gcode_emit) fn group_brims(
    traversal: &PreparedPostClassicTraversal,
    members: &[usize],
    plans: &[Option<BrimPlan>],
) -> Result<Vec<GroupBrim>, SliceError> {
    let owners = members
        .iter()
        .copied()
        .filter(|&i| plans[i].is_some())
        .collect::<Vec<_>>();
    if !traversal.resolved.views.full.process.print.combine_brims.0 || owners.len() < 2 {
        return Ok(owners.into_iter().map(GroupBrim::Single).collect());
    }
    fn root(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    let scale = traversal.scale;
    let template = plans[owners[0]].as_ref().unwrap();
    let spacing = super::scaled_f32(scale, f64::from(template.spacing))?;
    let contact = (f64::from(spacing) * 2.0) as i64;
    let resolution = scale.scaled_delta(0.0125);
    let mut parent = (0..owners.len()).collect::<Vec<_>>();
    // Print.cpp:2934-2946: contact is between owner areas, not skirt hulls.
    for i in 0..owners.len() {
        let expanded = crate::geometry::offset_expolygons(
            &plans[owners[i]].as_ref().unwrap().area,
            contact as f32,
            JoinType::Round,
            resolution,
        )
        .map_err(brim_geometry_error)?;
        for j in i + 1..owners.len() {
            if !crate::geometry::intersection_ex(
                &expanded,
                &plans[owners[j]].as_ref().unwrap().area,
            )
            .map_err(brim_geometry_error)?
            .is_empty()
            {
                let a = root(&mut parent, i);
                let b = root(&mut parent, j);
                parent[b] = a;
            }
        }
    }
    let mut components = std::collections::BTreeMap::<usize, Vec<usize>>::new();
    for (i, owner) in owners.into_iter().enumerate() {
        components
            .entry(root(&mut parent, i))
            .or_default()
            .push(owner);
    }
    let cleanup_resolution =
        scale.scaled_delta(traversal.resolved.views.full.process.print.resolution.0) as f32;
    let delta = cleanup_resolution.max(scale.scaled_delta(0.0001) as f32);
    components
        .into_values()
        .map(|owners| {
            if owners.len() == 1 {
                return Ok(GroupBrim::Single(owners[0]));
            }
            let area = owners
                .iter()
                .flat_map(|&i| plans[i].as_ref().unwrap().area.iter().cloned())
                .collect::<Vec<_>>();
            let area = union_expolygons(&area, &[]).map_err(brim_geometry_error)?;
            let area = offset2_ex(
                &area,
                delta,
                -delta,
                JoinType::Round,
                f64::from(cleanup_resolution),
            )
            .map_err(brim_geometry_error)?;
            let paths = crate::brims::ears::paths::from_area(area, spacing, resolution, scale)?;
            Ok(GroupBrim::Combined {
                owners,
                plan: BrimPlan {
                    area: Vec::new(),
                    global: false,
                    owner_collection: true,
                    paths,
                    covered_hull: Vec::new(),
                    width: template.width,
                    spacing: template.spacing,
                    height: template.height,
                    mm3_per_mm: template.mm3_per_mm,
                },
            })
        })
        .collect()
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
