use crate::geometry::ExPolygon;

use super::super::{compensation::PostCompensationPrintObject, region_slices::RegionSurface};

#[derive(Clone, Copy, Debug)]
pub(in crate::project_slice) struct Flow {
    pub(in crate::project_slice) width: f32,
    pub(in crate::project_slice) height: f32,
    pub(in crate::project_slice) spacing: f32,
    pub(in crate::project_slice) nozzle_diameter: f32,
    pub(in crate::project_slice) bridge: bool,
    pub(in crate::project_slice) mm3_per_mm: f64,
}

impl Flow {
    pub(in crate::project_slice) fn auto_infill_width(nozzle_diameter: f64) -> f64 {
        f64::from(1.125_f32 * nozzle_diameter as f32)
    }

    pub(in crate::project_slice) fn minimum_width(self) -> f32 {
        self.width + self.spacing
    }

    pub(in crate::project_slice) fn with_width(
        self,
        width: f32,
    ) -> Result<Self, crate::SliceError> {
        let rounded_rectangle_factor = (1.0 - 0.25 * std::f64::consts::PI) as f32;
        let spacing = width - self.height * rounded_rectangle_factor;
        let mm3_per_mm = f64::from(
            (f64::from(self.height)
                * (f64::from(width) - f64::from(self.height) * (1.0 - 0.25 * std::f64::consts::PI)))
                as f32,
        );
        if !spacing.is_finite() || spacing <= 0.0 || !mm3_per_mm.is_finite() || mm3_per_mm <= 0.0 {
            return Err(crate::SliceError::InvalidInput(
                "invalid smaller external perimeter flow".to_owned(),
            ));
        }
        Ok(Self {
            width,
            height: self.height,
            spacing,
            nozzle_diameter: self.nozzle_diameter,
            bridge: false,
            mm3_per_mm,
        })
    }
}

impl PartialEq for Flow {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && self.nozzle_diameter == other.nozzle_diameter
            && self.bridge == other.bridge
    }
}

#[derive(Clone, Copy, Debug)]
pub(in crate::project_slice) struct PerimeterFlows {
    pub(in crate::project_slice) perimeter_flow: Flow,
    pub(in crate::project_slice) ext_perimeter_flow: Flow,
    pub(in crate::project_slice) overhang_flow: Flow,
    pub(in crate::project_slice) solid_infill_flow: Flow,
}

#[derive(Debug)]
pub(in crate::project_slice) struct PreparedObjectFlows {
    pub(in crate::project_slice) layers: Vec<Option<PerimeterFlows>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum PerimeterDispatch {
    Classic,
    Arachne,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) struct RegionLayerIndex {
    pub(in crate::project_slice) region_index: usize,
    pub(in crate::project_slice) layer_index: usize,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::project_slice) struct PerimeterInputRecord {
    pub(in crate::project_slice) source_object_index: usize,
    pub(in crate::project_slice) transform_index: usize,
    pub(in crate::project_slice) planned_layer_index: usize,
    pub(in crate::project_slice) layer_id: usize,
    pub(in crate::project_slice) region_id: usize,
    pub(in crate::project_slice) compatible_region_ids: [usize; 1],
    pub(in crate::project_slice) current: RegionLayerIndex,
    pub(in crate::project_slice) lower_layer_index: Option<usize>,
    pub(in crate::project_slice) upper_layer_index: Option<usize>,
    pub(in crate::project_slice) upper_same_region: Option<RegionLayerIndex>,
    pub(in crate::project_slice) layer_height: f64,
    pub(in crate::project_slice) perimeter_flow: Flow,
    pub(in crate::project_slice) ext_perimeter_flow: Flow,
    pub(in crate::project_slice) overhang_flow: Flow,
    pub(in crate::project_slice) solid_infill_flow: Flow,
    pub(in crate::project_slice) spiral_mode: bool,
    pub(in crate::project_slice) model_rotation_rad: f64,
    pub(in crate::project_slice) dispatch: PerimeterDispatch,
}

pub(in crate::project_slice) struct PostPerimeterInputPrintObject {
    pub(in crate::project_slice) object: PostCompensationPrintObject,
    pub(in crate::project_slice) records: Vec<Option<PerimeterInputRecord>>,
}

impl PostPerimeterInputPrintObject {
    pub(in crate::project_slice) fn identity(&self) -> (usize, usize) {
        let (post_region, _) = self.object.as_parts();
        let (plan, _, _) = post_region.as_parts();
        (plan.source_object_index, plan.transform_index)
    }

    pub(in crate::project_slice) fn as_parts(
        &self,
    ) -> (
        &PostCompensationPrintObject,
        &[Option<PerimeterInputRecord>],
    ) {
        (&self.object, &self.records)
    }

    pub(in crate::project_slice) fn into_parts(
        self,
    ) -> (
        PostCompensationPrintObject,
        Vec<Option<PerimeterInputRecord>>,
    ) {
        (self.object, self.records)
    }

    pub(in crate::project_slice) fn current_surfaces(
        &self,
        record: &PerimeterInputRecord,
    ) -> &[RegionSurface] {
        self.region_surfaces(record.current)
    }

    pub(in crate::project_slice) fn current_slices(
        &self,
        record: &PerimeterInputRecord,
    ) -> &[ExPolygon] {
        self.layer_slices(record.current.layer_index)
    }

    /// Slices of every volume occurrence at the record's layer
    /// (`Layer::lslices` spans all instances of the print object).
    pub(in crate::project_slice) fn occurrence_slices(
        &self,
        record_index: usize,
    ) -> Vec<&[ExPolygon]> {
        let layer_index = self
            .records
            .get(record_index)
            .and_then(Option::as_ref)
            .map(|record| record.current.layer_index);
        let Some(layer_index) = layer_index else {
            return Vec::new();
        };
        let (post_region, _) = self.object.as_parts();
        if std::env::var("ARES_DUMP_BND").is_ok() {
            eprintln!(
                "OCC occurrences={} records={} layers0={}",
                post_region.volume_slices.len(),
                self.records.len(),
                post_region
                    .volume_slices
                    .first()
                    .map(|v| v.layers.len())
                    .unwrap_or(0)
            );
        }
        post_region
            .volume_slices
            .iter()
            .filter_map(|volume| volume.layers.get(layer_index).map(Vec::as_slice))
            .collect()
    }

    pub(in crate::project_slice) fn region_options(
        &self,
        record: &PerimeterInputRecord,
    ) -> &crate::RegionOptions {
        let (post_region, _) = self.object.as_parts();
        let (_, _, regions) = post_region.as_parts();
        regions[record.current.region_index].as_parts().1
    }

    pub(in crate::project_slice) fn lower_slices(
        &self,
        record: &PerimeterInputRecord,
    ) -> Option<&[ExPolygon]> {
        record
            .lower_layer_index
            .map(|layer_index| self.layer_slices(layer_index))
    }

    pub(in crate::project_slice) fn upper_slices(
        &self,
        record: &PerimeterInputRecord,
    ) -> Option<&[ExPolygon]> {
        record
            .upper_layer_index
            .map(|layer_index| self.layer_slices(layer_index))
    }

    pub(in crate::project_slice) fn upper_same_region_surfaces(
        &self,
        record: &PerimeterInputRecord,
    ) -> Option<&[RegionSurface]> {
        record
            .upper_same_region
            .map(|index| self.region_surfaces(index))
    }

    fn layer_slices(&self, layer_index: usize) -> &[ExPolygon] {
        self.object.as_parts().1[layer_index].as_slice()
    }

    fn region_surfaces(&self, index: RegionLayerIndex) -> &[RegionSurface] {
        let (post_region, _) = self.object.as_parts();
        let (_, _, regions) = post_region.as_parts();
        regions[index.region_index].as_parts().2[index.layer_index].surfaces()
    }
}
