use std::mem;

use crate::{
    FloatOrPercent, ObjectOptions, OrcaFloats, Project, SliceError,
    geometry::{CoordinateScale, ExPolygon, FillRule, union_ex},
    project::effective_config::types::BoundedResolvedProjectConfig,
};

use super::{
    PreparedPostRegions, ProjectBytes,
    elephant_foot::compensate_expolygons,
    hole_to_polyhole,
    load_stages::prepare_post_interlocking,
    region_slices::{PostRegion, PostRegionPrintObject, RegionSurface},
    slice_ordering::{make_slices, order_expolygons},
};

mod multi_region;
mod preflight;
mod prepare;
mod xy;

pub(in crate::project_slice) use prepare::apply_project_compensation;

use preflight::{
    PreparedLayerCompensation, PreparedObjectCompensation, prepare_object_compensation,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::project_slice) struct ValidatedTask22mConfig {
    pub(in crate::project_slice) compensation_mm: f64,
    pub(in crate::project_slice) xy_contour_mm: f64,
    pub(in crate::project_slice) xy_hole_mm: f64,
    pub(in crate::project_slice) compensation_layers: usize,
    pub(in crate::project_slice) raft_layers: i32,
    pub(in crate::project_slice) object_line_width: FloatOrPercent,
    pub(in crate::project_slice) closing_radius_mm: f64,
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

            let xy_hole_mm = options.xy_hole_compensation.0;
            let xy_contour_mm = options.xy_contour_compensation.0;
            if !xy_hole_mm.is_finite() {
                return Err(invalid("invalid Orca option xy_hole_compensation"));
            }
            if !xy_contour_mm.is_finite() {
                return Err(invalid("invalid Orca option xy_contour_compensation"));
            }

            Ok(ValidatedTask22mConfig {
                compensation_mm,
                xy_contour_mm,
                xy_hole_mm,
                compensation_layers,
                raft_layers: options.raft_layers.0,
                object_line_width: options.line_width,
                closing_radius_mm: options.slice_closing_radius.0,
            })
        })
        .collect()
}

fn invalid(message: &str) -> SliceError {
    SliceError::InvalidInput(message.to_owned())
}

type ValidateTask22mConfigs =
    fn(&[&ObjectOptions]) -> Result<Vec<ValidatedTask22mConfig>, SliceError>;
const _: ValidateTask22mConfigs = validate_task22m_configs;

pub(in crate::project_slice) struct PostCompensationPrintObject {
    post_regions: PostRegionPrintObject,
    volume_slices: Vec<RawBrimVolume>,
    lslices: Vec<Vec<ExPolygon>>,
    raw_slices: Vec<Vec<Vec<ExPolygon>>>,
}

#[derive(Clone)]
pub(in crate::project_slice) struct RawBrimVolume {
    pub(in crate::project_slice) source_volume_index: usize,
    pub(in crate::project_slice) volume_type: crate::ProjectVolumeType,
    pub(in crate::project_slice) first_layer: Vec<ExPolygon>,
}

impl PostCompensationPrintObject {
    pub(in crate::project_slice) fn volume_slices(&self) -> &[RawBrimVolume] {
        &self.volume_slices
    }

    pub(super) fn as_parts(&self) -> (&PostRegionPrintObject, &[Vec<ExPolygon>]) {
        (&self.post_regions, &self.lslices)
    }

    pub(super) fn into_parts(self) -> (PostRegionPrintObject, Vec<Vec<ExPolygon>>) {
        (self.post_regions, self.lslices)
    }

    pub(super) fn raw_slices(&self, region_index: usize, layer_index: usize) -> &[ExPolygon] {
        &self.raw_slices[region_index][layer_index]
    }

    /// Mutable `(post_regions, lslices)` access for
    /// `hole_to_polyhole::transform` (`PrintObject.cpp:189-295`), which
    /// rewrites region surface holes and the `lslices` clones in place.
    pub(super) fn polyhole_parts_mut(
        &mut self,
    ) -> (&mut PostRegionPrintObject, &mut Vec<Vec<ExPolygon>>) {
        (&mut self.post_regions, &mut self.lslices)
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
        painting,
    } = prepare_post_interlocking(project.into_source())?;
    let initial_layer_width = resolved.views.full.process.print.initial_layer_line_width;
    let nozzle_diameters = &resolved.views.full.project.print.nozzle_diameter;
    let objects = apply_project_compensation(
        objects,
        &resolved.objects,
        initial_layer_width,
        nozzle_diameters,
        (scale, painting.iter().map(|metadata| metadata.active)),
    )?;
    Ok(PreparedPostCompensation {
        project,
        resolved,
        config_block,
        scale,
        objects,
    })
}

fn validate_structures(objects: &[PostRegionPrintObject]) {
    for object in objects {
        let layer_count = object.plan.layers.len();
        for region in &object.regions {
            assert_eq!(
                region.layers.len(),
                layer_count,
                "Task 22M region layers must match the retained plan"
            );
        }
    }
}

fn apply_object_compensation(
    mut object: PostRegionPrintObject,
    prepared: PreparedObjectCompensation,
    nozzle_diameters: &OrcaFloats,
    scale: CoordinateScale,
    closing_epsilon: &mut Option<f32>,
) -> Result<PostCompensationPrintObject, SliceError> {
    let volume_slices = object
        .as_parts()
        .1
        .iter()
        .map(|volume| RawBrimVolume {
            source_volume_index: volume.source_volume_index(),
            volume_type: volume.volume_type(),
            first_layer: volume.first_layer().to_vec(),
        })
        .collect::<Vec<_>>();
    let backups = if object.regions.len() > 1 {
        multi_region::apply(&mut object.regions, &prepared, scale, closing_epsilon)?
    } else {
        apply_single_region_compensation(&mut object.regions, prepared, scale)?
    };

    let mut lslices = if object.plan.layers.is_empty() {
        Vec::new()
    } else {
        make_slices(&mut object).map_err(|_| geometry_error())?
    };
    for (layer_index, raw) in backups.into_iter().enumerate() {
        lslices[layer_index] = order_expolygons(raw);
    }
    // `PrintObjectSlice.cpp:847`: `_transform_hole_to_polyholes` runs
    // after `slice_volumes` (which includes this XY/EFC compensation,
    // `PrintObjectSlice.cpp:1250+`) and before `backup_untyped_slices`
    // (:851-867) — the `raw_slices` snapshot below therefore already
    // sees the polyhole points, matching `Layer::backup_untyped_slices`.
    let mut wrapped = PostCompensationPrintObject {
        post_regions: object,
        volume_slices: volume_slices.clone(),
        lslices,
        raw_slices: Vec::new(),
    };
    hole_to_polyhole::transform(&mut wrapped, nozzle_diameters, scale);
    let PostCompensationPrintObject {
        post_regions: object,
        volume_slices: _,
        lslices,
        ..
    } = wrapped;
    // Layer::backup_untyped_slices saves regions, not the elephant-foot lslices backup.
    let raw_slices = object
        .regions
        .iter()
        .map(|region| {
            region
                .layers
                .iter()
                .map(|layer| {
                    layer
                        .surfaces
                        .iter()
                        .map(|surface| surface.as_parts().1.clone())
                        .collect()
                })
                .collect()
        })
        .collect();
    Ok(PostCompensationPrintObject {
        post_regions: object,
        volume_slices,
        lslices,
        raw_slices,
    })
}

fn apply_single_region_compensation(
    regions: &mut [PostRegion],
    prepared: PreparedObjectCompensation,
    scale: CoordinateScale,
) -> Result<Vec<Vec<ExPolygon>>, SliceError> {
    let PreparedObjectCompensation {
        backup_len,
        xy_contour,
        xy_hole,
        layers,
        closing_radius_mm: _,
    } = prepared;
    let mut backups = vec![Vec::new(); backup_len];
    let [region] = regions else {
        return Ok(backups);
    };
    for (layer_index, prepared_layer) in layers.into_iter().enumerate() {
        if xy_contour == 0.0 && xy_hole == 0.0 && prepared_layer.is_none() {
            continue;
        }
        let surfaces = mem::take(&mut region.layers[layer_index].surfaces);
        let raw = surfaces
            .into_iter()
            .map(|surface| surface.into_parts().1)
            .collect::<Vec<_>>();
        let adjusted = xy::apply(&raw, xy_contour, xy_hole).map_err(|_| geometry_error())?;
        let compensated = if let Some(prepared_layer) = prepared_layer {
            backups[layer_index] = adjusted.clone();
            apply_elephant_foot(&adjusted, prepared_layer, scale)?
        } else {
            adjusted
        };
        region.layers[layer_index].surfaces = compensated
            .into_iter()
            .map(RegionSurface::internal)
            .collect();
    }
    Ok(backups)
}

fn apply_elephant_foot(
    source: &[ExPolygon],
    prepared: PreparedLayerCompensation,
    scale: CoordinateScale,
) -> Result<Vec<ExPolygon>, SliceError> {
    let compensated = compensate_expolygons(
        source,
        prepared.minimum_width_mm,
        prepared.compensation_mm,
        scale,
    )
    .map_err(|_| geometry_error())?;
    let paths = compensated
        .into_iter()
        .flat_map(|expolygon| {
            let (contour, holes) = expolygon.into_parts();
            std::iter::once(contour).chain(holes)
        })
        .collect::<Vec<_>>();
    union_ex(&paths, FillRule::NonZero).map_err(|_| geometry_error())
}

fn geometry_error() -> SliceError {
    invalid(
        "project elephant-foot compensation geometry is nonfinite or outside the supported Clipper range",
    )
}
