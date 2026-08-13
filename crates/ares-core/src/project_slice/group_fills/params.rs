mod locked;
mod projection;

use crate::{
    FloatOrPercent, ObjectOptions, OrcaFloats, RegionOptions, SliceError,
    geometry::{CoordinateScale, ExPolygon},
    project_slice::{
        layers::PlannedLayer,
        perimeters::types::{PerimeterInputRecord, PostPerimeterInputPrintObject},
        prepare_infill::{
            external_surfaces::PreparedPostExternalSurfaces,
            surface_type_detection::types::PreparedSurfaceTypeRecord,
        },
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

use super::{LockRegionParam, SurfaceFillParams};

pub(super) struct LayerContext<'a> {
    pub(super) planned: &'a PlannedLayer,
    pub(super) record: &'a PreparedSurfaceTypeRecord,
    pub(super) region_id: usize,
    pub(super) region: &'a RegionOptions,
    pub(super) object: &'a ObjectOptions,
    pub(super) nozzles: &'a OrcaFloats,
    pub(super) initial_layer_width: FloatOrPercent,
    pub(super) model_rotation_offset: f32,
    pub(super) scale: CoordinateScale,
}

pub(super) struct ProjectedLayer<'a> {
    pub(super) surfaces: &'a [RegionSurface],
    pub(super) params: Vec<Option<SurfaceFillParams>>,
    pub(super) region_id: usize,
    pub(super) no_overlap_expolygons: &'a [ExPolygon],
    pub(super) lock_region_param: LockRegionParam,
    pub(super) has_internal_voids: bool,
}

impl<'a> LayerContext<'a> {
    pub(super) fn new(
        prepared: &'a PreparedPostExternalSurfaces,
        prelude: &'a PostPerimeterInputPrintObject,
        record: &'a PreparedSurfaceTypeRecord,
        input: &'a PerimeterInputRecord,
        layer_index: usize,
    ) -> Self {
        let traversal = &prepared.predecessor.predecessor;
        let (compensated, _) = prelude.as_parts();
        let (post_regions, _) = compensated.as_parts();
        let (plan, _, _) = post_regions.as_parts();
        let resolved_object = traversal
            .resolved
            .objects
            .iter()
            .find(|object| object.source_object_index == input.source_object_index)
            .expect("fill grouping retains its resolved object");
        let object = &resolved_object.object;
        let model_rotation_offset = if prelude
            .region_options(input)
            .align_infill_direction_to_model
            .0
        {
            let (m00, m10) = resolved_object.print_objects[input.transform_index]
                .transform
                .first_xy_column();
            f64::from(m10 as f32).atan2(f64::from(m00 as f32)) as f32
        } else {
            0.0
        };
        Self {
            planned: &plan.layers[layer_index],
            record,
            region_id: input.current.region_index,
            region: prelude.region_options(input),
            object,
            nozzles: &traversal.resolved.views.full.project.print.nozzle_diameter,
            initial_layer_width: traversal
                .resolved
                .views
                .full
                .process
                .print
                .initial_layer_line_width,
            model_rotation_offset,
            scale: traversal.scale,
        }
    }
}

pub(super) fn project_layer(context: LayerContext<'_>) -> Result<ProjectedLayer<'_>, SliceError> {
    let mut current = projection::source_defaults();
    let mut projected = Vec::with_capacity(context.record.fill_surfaces.len());
    let mut lock_region_param = locked::Builder::default();
    let mut has_internal_voids = false;

    for surface in &context.record.fill_surfaces {
        if surface.as_parts().0 == RegionSurfaceKind::InternalVoid {
            has_internal_voids = true;
            projected.push(None);
            continue;
        }
        let params = projection::project_surface(&context, surface, &mut current)?;
        if let Some(params) = params {
            locked::append(&context, surface, params, &mut lock_region_param)?;
        }
        projected.push(params);
    }

    Ok(ProjectedLayer {
        surfaces: &context.record.fill_surfaces,
        params: projected,
        region_id: context.region_id,
        no_overlap_expolygons: &context.record.fill_no_overlap_expolygons,
        lock_region_param: lock_region_param.finish(),
        has_internal_voids,
    })
}
