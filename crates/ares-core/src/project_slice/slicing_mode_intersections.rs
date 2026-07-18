use crate::{
    ProcessSlicingMode, ProjectVolumeType, RegionOptions, SliceError,
    mesh_slicer::{LoopedLayer, SlicingMode, apply_slicing_mode},
    project::effective_config::types::ResolvedProjectObject,
};

use super::{
    layers::{PlannedLayer, PlannedPrintObject},
    looped_intersections::LoopedPrintObject,
};

const EPSILON: f64 = 1e-4;

pub(super) struct SlicingModeLayer {
    mode: SlicingMode,
    looped_layer: LoopedLayer,
}

impl SlicingModeLayer {
    pub(super) fn into_parts(self) -> (SlicingMode, LoopedLayer) {
        (self.mode, self.looped_layer)
    }

    #[cfg(test)]
    pub(super) const fn mode(&self) -> SlicingMode {
        self.mode
    }

    #[cfg(test)]
    pub(super) const fn looped_layer(&self) -> &LoopedLayer {
        &self.looped_layer
    }
}

pub(super) struct SlicingModeVolumeIntersections {
    source_volume_index: usize,
    volume_ordinal: u32,
    volume_type: ProjectVolumeType,
    layers: Vec<SlicingModeLayer>,
}

impl SlicingModeVolumeIntersections {
    pub(super) fn into_parts(self) -> (usize, u32, ProjectVolumeType, Vec<SlicingModeLayer>) {
        (
            self.source_volume_index,
            self.volume_ordinal,
            self.volume_type,
            self.layers,
        )
    }

    #[cfg(test)]
    pub(super) const fn source_volume_index(&self) -> usize {
        self.source_volume_index
    }

    #[cfg(test)]
    pub(super) const fn ordinal(&self) -> u32 {
        self.volume_ordinal
    }

    #[cfg(test)]
    pub(super) const fn volume_type(&self) -> ProjectVolumeType {
        self.volume_type
    }

    #[cfg(test)]
    pub(super) fn layers(&self) -> &[SlicingModeLayer] {
        &self.layers
    }

    #[cfg(test)]
    pub(super) fn set_ordinal_for_test(&mut self, ordinal: u32) {
        self.volume_ordinal = ordinal;
    }
}

pub(super) struct SlicingModePrintObject {
    plan: PlannedPrintObject,
    volumes: Vec<SlicingModeVolumeIntersections>,
}

impl SlicingModePrintObject {
    pub(super) fn into_parts(self) -> (PlannedPrintObject, Vec<SlicingModeVolumeIntersections>) {
        (self.plan, self.volumes)
    }

    #[cfg(test)]
    pub(super) const fn plan(&self) -> &PlannedPrintObject {
        &self.plan
    }

    #[cfg(test)]
    pub(super) fn volumes(&self) -> &[SlicingModeVolumeIntersections] {
        &self.volumes
    }

    #[cfg(test)]
    pub(super) fn volumes_mut(&mut self) -> &mut Vec<SlicingModeVolumeIntersections> {
        &mut self.volumes
    }
}

pub(super) fn apply_project_slicing_modes(
    objects: Vec<LoopedPrintObject>,
    resolved_objects: &[ResolvedProjectObject],
    spiral_mode: bool,
) -> Result<Vec<SlicingModePrintObject>, SliceError> {
    objects
        .into_iter()
        .map(|object| {
            let (plan, volumes) = object.into_parts();
            let resolved = resolved_objects
                .iter()
                .find(|resolved| resolved.source_object_index == plan.source_object_index)
                .expect("looped object must have resolved configuration");
            let base_mode = map_process_slicing_mode(resolved.object.slicing_mode);
            let volumes = volumes
                .into_iter()
                .map(|volume| {
                    let (source_volume_index, volume_ordinal, volume_type, layers) =
                        volume.into_parts();
                    let threshold = model_part_threshold(
                        &plan.layers,
                        resolved,
                        spiral_mode,
                        source_volume_index,
                        volume_type,
                    )?;
                    let layers = layers
                        .into_iter()
                        .enumerate()
                        .map(|(layer_index, looped_layer)| {
                            apply_layer_mode(looped_layer, layer_index, threshold, base_mode)
                        })
                        .collect();
                    Ok(SlicingModeVolumeIntersections {
                        source_volume_index,
                        volume_ordinal,
                        volume_type,
                        layers,
                    })
                })
                .collect::<Result<Vec<_>, SliceError>>()?;
            Ok(SlicingModePrintObject { plan, volumes })
        })
        .collect()
}

fn apply_layer_mode(
    mut looped_layer: LoopedLayer,
    layer_index: usize,
    threshold: Option<usize>,
    base_mode: SlicingMode,
) -> SlicingModeLayer {
    let mode = threshold.map_or(base_mode, |threshold| {
        if layer_index < threshold {
            base_mode
        } else {
            SlicingMode::PositiveLargestContour
        }
    });
    let raw_mode = match mode {
        SlicingMode::PositiveLargestContour => SlicingMode::Positive,
        mode => mode,
    };
    apply_slicing_mode(&mut looped_layer, raw_mode);
    SlicingModeLayer { mode, looped_layer }
}

pub(super) const fn map_process_slicing_mode(mode: ProcessSlicingMode) -> SlicingMode {
    match mode {
        ProcessSlicingMode::Regular => SlicingMode::Regular,
        ProcessSlicingMode::EvenOdd => SlicingMode::EvenOdd,
        ProcessSlicingMode::CloseHoles => SlicingMode::Positive,
    }
}

fn model_part_threshold(
    layers: &[PlannedLayer],
    resolved: &ResolvedProjectObject,
    spiral_mode: bool,
    source_volume_index: usize,
    volume_type: ProjectVolumeType,
) -> Result<Option<usize>, SliceError> {
    if !spiral_mode {
        return Ok(None);
    }
    match volume_type {
        ProjectVolumeType::ModelPart => {
            let [candidate] = resolved.layer_candidates.as_slice() else {
                panic!("looped object must have exactly one resolved layer candidate");
            };
            let region = &candidate
                .model_parts
                .iter()
                .find(|model_part| model_part.volume_index == source_volume_index)
                .expect("looped model part must have a resolved region")
                .region;
            spiral_bottom_threshold(layers, region).map(Some)
        }
        ProjectVolumeType::NegativeVolume | ProjectVolumeType::ParameterModifier => Ok(None),
        ProjectVolumeType::SupportEnforcer | ProjectVolumeType::SupportBlocker => {
            unreachable!("support volumes are filtered before raw intersections")
        }
    }
}

pub(super) fn spiral_bottom_threshold(
    layers: &[PlannedLayer],
    region: &RegionOptions,
) -> Result<usize, SliceError> {
    let mut threshold = usize::try_from(region.bottom_shell_layers.0)
        .map_err(|_| invalid_option("bottom_shell_layers"))?;
    let thickness = region.bottom_shell_thickness.0;
    if !thickness.is_finite() || thickness < 0.0 {
        return Err(invalid_option("bottom_shell_thickness"));
    }
    let boundary = thickness - EPSILON;
    while threshold < layers.len() && f64::from(layers[threshold].slice_z as f32) < boundary {
        threshold += 1;
    }
    Ok(threshold)
}

fn invalid_option(key: &str) -> SliceError {
    SliceError::InvalidInput(format!("invalid Orca option {key}"))
}
