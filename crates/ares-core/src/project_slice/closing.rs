use crate::{
    ProjectVolumeType, SliceError,
    geometry::{ClipperError, CoordinateScale, ExPolygon, JoinType, offset2_ex},
    mesh_slicer::SlicingMode,
    project::effective_config::types::ResolvedProjectObject,
};

use super::{layers::PlannedPrintObject, pre_closing_unions::PreClosingPrintObject};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ClosingDeltas {
    pub(super) outward: f32,
    pub(super) inward: f32,
}

pub(super) struct PostClosingLayer {
    mode: SlicingMode,
    expolygons: Vec<ExPolygon>,
}

impl PostClosingLayer {
    #[cfg(test)]
    pub(super) fn new(mode: SlicingMode, expolygons: Vec<ExPolygon>) -> Self {
        Self { mode, expolygons }
    }

    pub(super) fn into_parts(self) -> (SlicingMode, Vec<ExPolygon>) {
        (self.mode, self.expolygons)
    }

    #[cfg(any(test, feature = "task22g-browser-oracle"))]
    pub(super) const fn mode(&self) -> SlicingMode {
        self.mode
    }

    #[cfg(any(test, feature = "task22g-browser-oracle"))]
    pub(super) fn expolygons(&self) -> &[ExPolygon] {
        &self.expolygons
    }
}

pub(super) struct PostClosingVolume {
    source_volume_index: usize,
    volume_ordinal: u32,
    volume_type: ProjectVolumeType,
    layers: Vec<PostClosingLayer>,
}

impl PostClosingVolume {
    #[cfg(test)]
    pub(super) fn new(
        source_volume_index: usize,
        volume_ordinal: u32,
        volume_type: ProjectVolumeType,
        layers: Vec<PostClosingLayer>,
    ) -> Self {
        Self {
            source_volume_index,
            volume_ordinal,
            volume_type,
            layers,
        }
    }

    pub(super) fn into_parts(self) -> (usize, u32, ProjectVolumeType, Vec<PostClosingLayer>) {
        (
            self.source_volume_index,
            self.volume_ordinal,
            self.volume_type,
            self.layers,
        )
    }

    #[cfg(any(test, feature = "task22g-browser-oracle"))]
    pub(super) const fn source_volume_index(&self) -> usize {
        self.source_volume_index
    }

    #[cfg(any(test, feature = "task22g-browser-oracle"))]
    pub(super) const fn ordinal(&self) -> u32 {
        self.volume_ordinal
    }

    #[cfg(any(test, feature = "task22g-browser-oracle"))]
    pub(super) const fn volume_type(&self) -> ProjectVolumeType {
        self.volume_type
    }

    #[cfg(any(test, feature = "task22g-browser-oracle"))]
    pub(super) fn layers(&self) -> &[PostClosingLayer] {
        &self.layers
    }
}

pub(super) struct PostClosingPrintObject {
    plan: PlannedPrintObject,
    volumes: Vec<PostClosingVolume>,
}

impl PostClosingPrintObject {
    #[cfg(test)]
    pub(super) fn new(plan: PlannedPrintObject, volumes: Vec<PostClosingVolume>) -> Self {
        Self { plan, volumes }
    }

    pub(super) fn into_parts(self) -> (PlannedPrintObject, Vec<PostClosingVolume>) {
        (self.plan, self.volumes)
    }

    #[cfg(any(test, feature = "task22g-browser-oracle"))]
    pub(super) const fn plan(&self) -> &PlannedPrintObject {
        &self.plan
    }

    #[cfg(any(test, feature = "task22g-browser-oracle"))]
    pub(super) fn volumes(&self) -> &[PostClosingVolume] {
        &self.volumes
    }
}

pub(super) fn apply_project_closing(
    objects: Vec<PreClosingPrintObject>,
    resolved_objects: &[ResolvedProjectObject],
    scale: CoordinateScale,
) -> Result<Vec<PostClosingPrintObject>, SliceError> {
    objects
        .into_iter()
        .map(|object| {
            let (plan, volumes) = object.into_parts();
            let resolved = resolved_objects
                .iter()
                .find(|resolved| resolved.source_object_index == plan.source_object_index)
                .expect("pre-closing object must have resolved configuration");
            let deltas = closing_deltas(resolved.object.slice_closing_radius.0, scale)?;
            let volumes = volumes
                .into_iter()
                .map(|volume| {
                    let (source_volume_index, volume_ordinal, volume_type, layers) =
                        volume.into_parts();
                    let layers = layers
                        .into_iter()
                        .map(|layer| {
                            let (mode, expolygons) = layer.into_parts();
                            Ok(PostClosingLayer {
                                mode,
                                expolygons: close_expolygons(expolygons, deltas)?,
                            })
                        })
                        .collect::<Result<Vec<_>, SliceError>>()?;
                    Ok(PostClosingVolume {
                        source_volume_index,
                        volume_ordinal,
                        volume_type,
                        layers,
                    })
                })
                .collect::<Result<Vec<_>, SliceError>>()?;
            Ok(PostClosingPrintObject { plan, volumes })
        })
        .collect()
}

pub(super) fn closing_deltas(
    radius_mm: f64,
    scale: CoordinateScale,
) -> Result<Option<ClosingDeltas>, SliceError> {
    if !radius_mm.is_finite() || radius_mm < 0.0 {
        return Err(invalid_radius());
    }
    let closing_radius = radius_mm as f32;
    let outward = (f64::from(closing_radius) / scale.factor()) as f32;
    let inward = (-f64::from(closing_radius - 0.0) / scale.factor()) as f32;
    if !outward.is_finite() || !inward.is_finite() {
        return Err(invalid_radius());
    }
    Ok((outward > 0.0 && inward < 0.0).then_some(ClosingDeltas { outward, inward }))
}

pub(super) fn close_expolygons(
    expolygons: Vec<ExPolygon>,
    deltas: Option<ClosingDeltas>,
) -> Result<Vec<ExPolygon>, SliceError> {
    let Some(deltas) = deltas else {
        return Ok(expolygons);
    };
    offset2_ex(
        &expolygons,
        deltas.outward,
        deltas.inward,
        JoinType::Miter,
        3.0,
    )
    .map_err(map_clipper_error)
}

fn invalid_radius() -> SliceError {
    SliceError::InvalidInput("invalid Orca option slice_closing_radius".to_owned())
}

fn map_clipper_error(error: ClipperError) -> SliceError {
    match error {
        ClipperError::CoordinateOutOfRange => SliceError::InvalidInput(
            "project closing polygon coordinate is outside the supported Clipper range".to_owned(),
        ),
    }
}
