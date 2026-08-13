use std::{
    cmp::Ordering,
    collections::{BTreeMap, btree_map::Entry},
};

use crate::{
    ProcessInfillPattern, SliceError,
    geometry::ExPolygon,
    project_slice::{
        perimeters::{
            flow::{FillFlowContext, resolve_configured_fill_flow},
            types::Flow,
        },
        region_slices::RegionSurface,
    },
};

use super::{
    LayerContext,
    projection::{configured, flow_role, is_solid},
};
use crate::project_slice::group_fills::{
    LockDensityParam, LockFlowParam, LockRegionParam, SurfaceFillParams,
};

#[derive(Default)]
pub(super) struct Builder {
    skin_density_params: BTreeMap<SourceOrd<f32>, LockDensityParam>,
    skeleton_density_params: BTreeMap<SourceOrd<f32>, LockDensityParam>,
    skin_flow_params: BTreeMap<SourceOrd<f64>, LockFlowParam>,
    skeleton_flow_params: BTreeMap<SourceOrd<f64>, LockFlowParam>,
}

pub(super) fn append(
    context: &LayerContext<'_>,
    surface: &RegionSurface,
    params: SurfaceFillParams,
    lock: &mut Builder,
) -> Result<(), SliceError> {
    if params.pattern != configured(ProcessInfillPattern::LockedZag) {
        return Ok(());
    }
    let (kind, expolygon, thickness, _, _, _) = surface.as_parts();
    let actual_height = if thickness == -1.0 {
        context.planned.height
    } else {
        thickness
    };
    let flow_context = FillFlowContext::new(
        context.planned,
        actual_height,
        context.initial_layer_width,
        context.region,
        context.object,
        context.nozzles,
    );
    let selected_role = flow_role(kind, is_solid(kind));
    let skin = if params.bridge {
        params.flow
    } else {
        resolve_configured_fill_flow(
            flow_context,
            selected_role,
            context.region.skin_infill_line_width,
        )?
    };
    let skeleton = if params.bridge {
        params.flow
    } else {
        resolve_configured_fill_flow(
            flow_context,
            selected_role,
            context.region.skeleton_infill_line_width,
        )?
    };
    append_flow(&mut lock.skin_flow_params, skin, expolygon);
    append_flow(&mut lock.skeleton_flow_params, skeleton, expolygon);
    append_density(
        &mut lock.skin_density_params,
        (0.01 * context.region.skin_infill_density.0) as f32,
        expolygon,
    );
    append_density(
        &mut lock.skeleton_density_params,
        (0.01 * context.region.skeleton_infill_density.0) as f32,
        expolygon,
    );
    Ok(())
}

impl Builder {
    pub(super) fn finish(self) -> LockRegionParam {
        LockRegionParam {
            skin_density_params: self.skin_density_params.into_values().collect(),
            skeleton_density_params: self.skeleton_density_params.into_values().collect(),
            skin_flow_params: self.skin_flow_params.into_values().collect(),
            skeleton_flow_params: self.skeleton_flow_params.into_values().collect(),
        }
    }
}

fn append_density(
    entries: &mut BTreeMap<SourceOrd<f32>, LockDensityParam>,
    density: f32,
    expolygon: &ExPolygon,
) {
    match entries.entry(SourceOrd(density)) {
        Entry::Occupied(mut entry) => entry.get_mut().expolygons.push(expolygon.clone()),
        Entry::Vacant(entry) => {
            entry.insert(LockDensityParam {
                density,
                expolygons: vec![expolygon.clone()],
            });
        }
    }
}

fn append_flow(
    entries: &mut BTreeMap<SourceOrd<f64>, LockFlowParam>,
    flow: Flow,
    expolygon: &ExPolygon,
) {
    match entries.entry(SourceOrd(flow.mm3_per_mm)) {
        Entry::Occupied(mut entry) => entry.get_mut().expolygons.push(expolygon.clone()),
        Entry::Vacant(entry) => {
            entry.insert(LockFlowParam {
                flow,
                expolygons: vec![expolygon.clone()],
            });
        }
    }
}

#[derive(Clone, Copy)]
struct SourceOrd<T>(T);

impl<T: Copy + PartialOrd> PartialEq for SourceOrd<T> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<T: Copy + PartialOrd> Eq for SourceOrd<T> {}

impl<T: Copy + PartialOrd> PartialOrd for SourceOrd<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Copy + PartialOrd> Ord for SourceOrd<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0 < other.0 {
            Ordering::Less
        } else if self.0 > other.0 {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}
