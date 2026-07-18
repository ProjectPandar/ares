use crate::{
    ProjectVolumeType, SliceError,
    geometry::{ClipperError, ExPolygon, FillRule, Polygon, union_ex},
    mesh_slicer::SlicingMode,
};

use super::{
    layers::PlannedPrintObject,
    slicing_mode_intersections::{SlicingModePrintObject, SlicingModeVolumeIntersections},
};

pub(super) struct PreClosingLayer {
    mode: SlicingMode,
    expolygons: Vec<ExPolygon>,
}

impl PreClosingLayer {
    pub(super) fn into_parts(self) -> (SlicingMode, Vec<ExPolygon>) {
        (self.mode, self.expolygons)
    }

    #[cfg(test)]
    pub(super) const fn mode(&self) -> SlicingMode {
        self.mode
    }

    #[cfg(test)]
    pub(super) fn expolygons(&self) -> &[ExPolygon] {
        &self.expolygons
    }
}

pub(super) struct PreClosingVolume {
    source_volume_index: usize,
    volume_ordinal: u32,
    volume_type: ProjectVolumeType,
    layers: Vec<PreClosingLayer>,
}

impl PreClosingVolume {
    pub(super) fn into_parts(self) -> (usize, u32, ProjectVolumeType, Vec<PreClosingLayer>) {
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
    pub(super) fn layers(&self) -> &[PreClosingLayer] {
        &self.layers
    }
}

pub(super) struct PreClosingPrintObject {
    plan: PlannedPrintObject,
    volumes: Vec<PreClosingVolume>,
}

impl PreClosingPrintObject {
    pub(super) fn into_parts(self) -> (PlannedPrintObject, Vec<PreClosingVolume>) {
        (self.plan, self.volumes)
    }

    #[cfg(test)]
    pub(super) const fn plan(&self) -> &PlannedPrintObject {
        &self.plan
    }

    #[cfg(test)]
    pub(super) fn volumes(&self) -> &[PreClosingVolume] {
        &self.volumes
    }
}

pub(super) fn apply_project_pre_closing_unions(
    objects: Vec<SlicingModePrintObject>,
) -> Result<Vec<PreClosingPrintObject>, SliceError> {
    objects
        .into_iter()
        .map(|object| {
            let (plan, volumes) = object.into_parts();
            let mut volumes = volumes
                .into_iter()
                .map(SlicingModeVolumeIntersections::into_parts)
                .collect::<Vec<_>>();
            volumes.sort_by_key(|(_, ordinal, _, _)| *ordinal);
            assert!(
                volumes.windows(2).all(|pair| pair[0].1 != pair[1].1),
                "duplicate pre-closing volume ordinal"
            );
            let volumes = volumes
                .into_iter()
                .map(
                    |(source_volume_index, volume_ordinal, volume_type, layers)| {
                        let layers = layers
                            .into_iter()
                            .map(|layer| {
                                let (mode, looped_layer) = layer.into_parts();
                                let expolygons =
                                    union_layer_polygons(mode, looped_layer.polygons())?;
                                Ok(PreClosingLayer { mode, expolygons })
                            })
                            .collect::<Result<Vec<_>, SliceError>>()?;
                        Ok(PreClosingVolume {
                            source_volume_index,
                            volume_ordinal,
                            volume_type,
                            layers,
                        })
                    },
                )
                .collect::<Result<Vec<_>, SliceError>>()?;
            Ok(PreClosingPrintObject { plan, volumes })
        })
        .collect()
}

pub(super) fn union_layer_polygons(
    mode: SlicingMode,
    polygons: &[Polygon],
) -> Result<Vec<ExPolygon>, SliceError> {
    union_ex(polygons, fill_rule_for_mode(mode)).map_err(map_clipper_error)
}

pub(super) const fn fill_rule_for_mode(mode: SlicingMode) -> FillRule {
    match mode {
        SlicingMode::Regular | SlicingMode::Positive => FillRule::NonZero,
        SlicingMode::EvenOdd => FillRule::EvenOdd,
        SlicingMode::PositiveLargestContour => FillRule::Positive,
    }
}

fn map_clipper_error(error: ClipperError) -> SliceError {
    match error {
        ClipperError::CoordinateOutOfRange => SliceError::InvalidInput(
            "project pre-closing polygon coordinate is outside the supported Clipper range"
                .to_owned(),
        ),
    }
}
