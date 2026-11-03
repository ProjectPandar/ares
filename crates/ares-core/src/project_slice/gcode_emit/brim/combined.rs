//! Print.cpp:4297-4389: per-instance group brim assembly with the
//! `combine_brims` contact union.
use super::{BrimPlan, brim_geometry_error};
use crate::{
    SliceError,
    geometry::{JoinType, offset2_ex, union_expolygons},
    project_slice::{
        gcode_emit::motion::EmitState, perimeters::classic::traversal::PreparedPostClassicTraversal,
    },
};

pub(in crate::project_slice::gcode_emit) enum GroupBrim {
    /// Print.cpp:4327-4336: one instance's own per-instance brim
    /// (`m_brimMapByInstance` entry).
    Instance { owner: usize, copy: usize },
    /// Print.cpp:4352-4389: a contacting cluster — union area, cleanup,
    /// one combined infill.
    Combined {
        instances: Vec<(usize, usize)>,
        plan: BrimPlan,
    },
}

impl GroupBrim {
    /// Emission match for the visiting instance chunk
    /// (`generate_object_brim`, GCode.cpp:6065-6071).
    pub(in crate::project_slice::gcode_emit) fn matches(&self, instance: (usize, usize)) -> bool {
        match self {
            Self::Instance { owner, copy } => (*owner, *copy) == instance,
            Self::Combined { instances, .. } => instances.contains(&instance),
        }
    }

    /// `m_objsWithBrim` (GCode.cpp:6056-6064): a brim is done once any of
    /// its instances has emitted it; emitting erases every instance.
    pub(in crate::project_slice::gcode_emit) fn emitted(&self, state: &EmitState) -> bool {
        match self {
            Self::Instance { owner, copy } => {
                state.emitted_brim_instances.contains(&(*owner, *copy))
            }
            Self::Combined { instances, .. } => instances
                .iter()
                .all(|instance| state.emitted_brim_instances.contains(instance)),
        }
    }

    pub(in crate::project_slice::gcode_emit) fn mark_done(&self, state: &mut EmitState) {
        match self {
            Self::Instance { owner, copy } => {
                state.emitted_brim_instances.insert((*owner, *copy));
            }
            Self::Combined { instances, .. } => {
                for instance in instances {
                    state.emitted_brim_instances.insert(*instance);
                }
            }
        }
    }

    pub(in crate::project_slice::gcode_emit) fn plan<'a>(
        &'a self,
        plans: &'a [Option<BrimPlan>],
    ) -> &'a BrimPlan {
        match self {
            Self::Instance { owner, .. } => plans[*owner].as_ref().unwrap(),
            Self::Combined { plan, .. } => plan,
        }
    }
}

pub(in crate::project_slice::gcode_emit) fn group_brims(
    traversal: &PreparedPostClassicTraversal,
    members: &[(usize, usize)],
    plans: &[Option<BrimPlan>],
) -> Result<Vec<GroupBrim>, SliceError> {
    let print = &traversal.resolved.views.full.process.print;
    // Print.cpp:4300-4305: brim instances are the group members whose
    // per-instance brim (m_brimMapByInstance) is non-empty.
    let instances = members
        .iter()
        .copied()
        .filter(|&(object, copy)| {
            plans[object].as_ref().is_some_and(|plan| {
                plan.instances
                    .iter()
                    .any(|instance| instance.copy == copy && !instance.paths.is_empty())
            })
        })
        .collect::<Vec<_>>();
    if !print.combine_brims.0 || instances.len() < 2 {
        return Ok(instances
            .into_iter()
            .map(|(owner, copy)| GroupBrim::Instance { owner, copy })
            .collect());
    }
    // Print.cpp:4352-4363: contact is between per-instance areas
    // (m_objectBrimAreasByInstance), offset by twice the brim spacing.
    let spacing = super::scaled_f32(
        traversal.scale,
        f64::from(template(plans, &instances).spacing),
    )?;
    let contact = (f64::from(spacing) * 2.0) as i64;
    let resolution = traversal.scale.scaled_delta(0.0125);
    let mut parent = (0..instances.len()).collect::<Vec<_>>();
    for i in 0..instances.len() {
        let Some(area_i) = instance_area(plans, instances[i]) else {
            continue;
        };
        let expanded =
            crate::geometry::offset_expolygons(area_i, contact as f32, JoinType::Round, resolution)
                .map_err(brim_geometry_error)?;
        for (j, &instance_j) in instances.iter().enumerate().skip(i + 1) {
            let Some(area_j) = instance_area(plans, instance_j) else {
                continue;
            };
            if !crate::geometry::intersection_ex(&expanded, area_j)
                .map_err(brim_geometry_error)?
                .is_empty()
            {
                let a = root(&mut parent, i);
                let b = root(&mut parent, j);
                parent[b] = a;
            }
        }
    }
    let mut components = std::collections::BTreeMap::<usize, Vec<(usize, usize)>>::new();
    for (i, instance) in instances.iter().copied().enumerate() {
        components
            .entry(root(&mut parent, i))
            .or_default()
            .push(instance);
    }
    let cleanup_resolution = traversal.scale.scaled_delta(print.resolution.0) as f32;
    let delta = cleanup_resolution.max(traversal.scale.scaled_delta(0.0001) as f32);
    components
        .into_values()
        .map(|cluster| {
            if cluster.len() == 1 {
                let (owner, copy) = cluster[0];
                return Ok(GroupBrim::Instance { owner, copy });
            }
            // Print.cpp:4366-4382: union the cluster's per-instance
            // areas, clean up, and build one combined infill.
            let area = cluster
                .iter()
                .filter_map(|&instance| instance_area(plans, instance))
                .flat_map(|area| area.iter().cloned())
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
            let template = template(plans, &cluster);
            let paths =
                crate::brims::ears::paths::from_area(area, spacing, resolution, traversal.scale)?;
            Ok(GroupBrim::Combined {
                instances: cluster,
                plan: BrimPlan {
                    area: Vec::new(),
                    instances: Vec::new(),
                    instance_areas: Vec::new(),
                    paths,
                    covered_hull: Vec::new(),
                    width: template.width,
                    spacing: template.spacing,
                    height: template.height,
                    mm3_per_mm: template.mm3_per_mm,
                    plate_origin: template.plate_origin,
                },
            })
        })
        .collect()
}

fn template<'a>(plans: &'a [Option<BrimPlan>], instances: &[(usize, usize)]) -> &'a BrimPlan {
    instances
        .iter()
        .map(|&(owner, _)| plans[owner].as_ref().unwrap())
        .next()
        .unwrap()
}

fn instance_area(
    plan: &[Option<BrimPlan>],
    instance: (usize, usize),
) -> Option<&[crate::geometry::ExPolygon]> {
    plan[instance.0]
        .as_ref()?
        .instance_areas
        .iter()
        .find(|area| area.copy == instance.1)
        .map(|area| area.area.as_slice())
}

fn root(parent: &mut [usize], mut i: usize) -> usize {
    while parent[i] != i {
        parent[i] = parent[parent[i]];
        i = parent[i];
    }
    i
}
