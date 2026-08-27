use std::mem;

use crate::{
    FloatOrPercent, ObjectOptions, OrcaFloats, Project, SliceError,
    geometry::{CoordinateScale, ExPolygon, FillRule, union_ex},
    project::effective_config::types::{BoundedResolvedProjectConfig, ResolvedProjectObject},
};

use super::{
    PreparedPostRegions, ProjectBytes,
    elephant_foot::compensate_expolygons,
    prepare_post_conical_overhang,
    region_slices::{PostRegionPrintObject, RegionSurface},
    slice_ordering::{make_single_region_slices, order_expolygons},
};

mod preflight;

use preflight::{PreparedObjectCompensation, prepare_object_compensation};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::project_slice) struct ValidatedTask22mConfig {
    pub(in crate::project_slice) compensation_mm: f64,
    pub(in crate::project_slice) compensation_layers: usize,
    pub(in crate::project_slice) raft_layers: i32,
    pub(in crate::project_slice) object_line_width: FloatOrPercent,
}

pub(in crate::project_slice) fn validate_task22m_configs(
    objects: &[&ObjectOptions],
) -> Result<Vec<ValidatedTask22mConfig>, SliceError> {
    objects
        .iter()
        .map(|options| {
            let compensation_mm = options.elefant_foot_compensation.0;
            if !compensation_mm.is_finite() || compensation_mm < 0.0 {
                return Err(invalid("invalid Orca option elefant_foot_compensation"));
            }

            let raw_layers = options.elefant_foot_compensation_layers.0;
            if raw_layers <= 0 {
                return Err(invalid(
                    "invalid Orca option elefant_foot_compensation_layers",
                ));
            }
            let compensation_layers = usize::try_from(raw_layers)
                .expect("a positive i32 compensation layer count must fit usize");

            if options.xy_hole_compensation.0 != 0.0 {
                return Err(unsupported("xy_hole_compensation"));
            }
            if options.xy_contour_compensation.0 != 0.0 {
                return Err(unsupported("xy_contour_compensation"));
            }

            Ok(ValidatedTask22mConfig {
                compensation_mm,
                compensation_layers,
                raft_layers: options.raft_layers.0,
                object_line_width: options.line_width,
            })
        })
        .collect()
}

fn invalid(message: &str) -> SliceError {
    SliceError::InvalidInput(message.to_owned())
}

fn unsupported(key: &str) -> SliceError {
    SliceError::UnsupportedProjectFeature(key.to_owned())
}

type ValidateTask22mConfigs =
    fn(&[&ObjectOptions]) -> Result<Vec<ValidatedTask22mConfig>, SliceError>;
const _: ValidateTask22mConfigs = validate_task22m_configs;

pub(in crate::project_slice) struct PostCompensationPrintObject {
    post_regions: PostRegionPrintObject,
    lslices: Vec<Vec<ExPolygon>>,
}

impl PostCompensationPrintObject {
    pub(super) fn as_parts(&self) -> (&PostRegionPrintObject, &[Vec<ExPolygon>]) {
        (&self.post_regions, &self.lslices)
    }

    pub(super) fn into_parts(self) -> (PostRegionPrintObject, Vec<Vec<ExPolygon>>) {
        (self.post_regions, self.lslices)
    }

    #[cfg(test)]
    pub(in crate::project_slice) fn as_parts_mut(
        &mut self,
    ) -> (&mut PostRegionPrintObject, &mut Vec<Vec<ExPolygon>>) {
        (&mut self.post_regions, &mut self.lslices)
    }
}

pub(super) struct PreparedPostCompensation {
    pub(super) project: Project,
    pub(super) resolved: BoundedResolvedProjectConfig,
    pub(super) config_block: Option<Vec<u8>>,
    pub(super) scale: CoordinateScale,
    pub(super) objects: Vec<PostCompensationPrintObject>,
}

pub(super) fn prepare_post_compensation<'a>(
    project: impl ProjectBytes<'a>,
) -> Result<PreparedPostCompensation, SliceError> {
    let PreparedPostRegions {
        project,
        resolved,
        config_block,
        scale,
        objects,
    } = prepare_post_conical_overhang(project.into_source())?;
    let initial_layer_width = resolved.views.full.process.print.initial_layer_line_width;
    let nozzle_diameters = &resolved.views.full.project.print.nozzle_diameter;
    let objects = apply_project_compensation(
        objects,
        &resolved.objects,
        initial_layer_width,
        nozzle_diameters,
        scale,
    )?;
    Ok(PreparedPostCompensation {
        project,
        resolved,
        config_block,
        scale,
        objects,
    })
}

pub(in crate::project_slice) fn apply_project_compensation(
    objects: Vec<PostRegionPrintObject>,
    resolved_objects: &[ResolvedProjectObject],
    initial_layer_width: FloatOrPercent,
    nozzle_diameters: &OrcaFloats,
    scale: CoordinateScale,
) -> Result<Vec<PostCompensationPrintObject>, SliceError> {
    let contexts = resolved_objects
        .iter()
        .flat_map(|resolved| {
            resolved
                .print_objects
                .iter()
                .enumerate()
                .map(move |(transform_index, _)| (resolved, transform_index))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        objects.len(),
        contexts.len(),
        "Task 22M objects must match resolved print-object contexts"
    );
    for (object, (resolved, transform_index)) in objects.iter().zip(&contexts) {
        assert_eq!(
            object.plan.source_object_index,
            resolved.source_object_index
        );
        assert_eq!(object.plan.transform_index, *transform_index);
    }

    let options = contexts
        .iter()
        .map(|(resolved, _)| &resolved.object)
        .collect::<Vec<_>>();
    let configs = validate_task22m_configs(&options)?;
    validate_structures(&objects)?;
    let prepared = objects
        .iter()
        .zip(&configs)
        .map(|(object, config)| {
            prepare_object_compensation(
                object,
                *config,
                initial_layer_width,
                nozzle_diameters,
                scale,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    objects
        .into_iter()
        .zip(prepared)
        .map(|(object, prepared)| apply_object_compensation(object, prepared, scale))
        .collect()
}

fn validate_structures(objects: &[PostRegionPrintObject]) -> Result<(), SliceError> {
    for object in objects {
        let layer_count = object.plan.layers.len();
        for region in &object.regions {
            assert_eq!(
                region.layers.len(),
                layer_count,
                "Task 22M region layers must match the retained plan"
            );
        }
        if layer_count != 0 && object.regions.len() > 1 {
            return Err(unsupported("multi_region_layer_slices"));
        }
    }
    Ok(())
}

fn apply_object_compensation(
    mut object: PostRegionPrintObject,
    prepared: PreparedObjectCompensation,
    scale: CoordinateScale,
) -> Result<PostCompensationPrintObject, SliceError> {
    let mut backups = (0..prepared.backup_len)
        .map(|_| Vec::new())
        .collect::<Vec<Vec<ExPolygon>>>();
    if let [region] = object.regions.as_mut_slice() {
        for (layer_index, prepared_layer) in prepared.layers.into_iter().enumerate() {
            let Some(prepared_layer) = prepared_layer else {
                continue;
            };
            let surfaces = mem::take(&mut region.layers[layer_index].surfaces);
            let raw = surfaces
                .into_iter()
                .map(|surface| surface.into_parts().1)
                .collect::<Vec<_>>();
            let compensated = compensate_expolygons(
                &raw,
                prepared_layer.minimum_width_mm,
                prepared_layer.compensation_mm,
                scale,
            )
            .map_err(|_| geometry_error())?;
            let mut paths = Vec::new();
            for expolygon in compensated {
                let (contour, holes) = expolygon.into_parts();
                paths.push(contour);
                paths.extend(holes);
            }
            let compensated = union_ex(&paths, FillRule::NonZero).map_err(|_| geometry_error())?;
            backups[layer_index] = raw;
            region.layers[layer_index].surfaces = compensated
                .into_iter()
                .map(RegionSurface::internal)
                .collect();
        }
    }

    let mut lslices = if object.plan.layers.is_empty() {
        Vec::new()
    } else {
        make_single_region_slices(&object)
    };
    for (layer_index, raw) in backups.into_iter().enumerate() {
        lslices[layer_index] = order_expolygons(raw);
    }
    Ok(PostCompensationPrintObject {
        post_regions: object,
        lslices,
    })
}

fn geometry_error() -> SliceError {
    invalid(
        "project elephant-foot compensation geometry is nonfinite or outside the supported Clipper range",
    )
}

type ApplyProjectCompensation = fn(
    Vec<PostRegionPrintObject>,
    &[ResolvedProjectObject],
    FloatOrPercent,
    &OrcaFloats,
    CoordinateScale,
) -> Result<Vec<PostCompensationPrintObject>, SliceError>;
const _: ApplyProjectCompensation = apply_project_compensation;
