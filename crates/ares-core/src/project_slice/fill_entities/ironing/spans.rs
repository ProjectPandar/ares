use crate::{ObjectOptions, OrcaFloats, ProcessRegionSourceOptions, SliceError};
use crate::{
    geometry::{CoordinateScale, ExPolygon},
    project_slice::{
        layers::PlannedLayer, perimeters::types::PostPerimeterInputPrintObject,
        prepare_infill::external_surfaces::PreparedPostExternalSurfaces,
    },
};

use super::{Input, LayerFillEntities, SpanContext, append, span_sort_key};

pub(crate) struct SpanGeneration<'a> {
    pub(crate) output: &'a mut LayerFillEntities,
    pub(crate) prepared: &'a PreparedPostExternalSurfaces,
    pub(crate) object_index: usize,
    pub(crate) layer_index: usize,
    pub(crate) prelude: &'a PostPerimeterInputPrintObject,
    pub(crate) layer: &'a PlannedLayer,
    pub(crate) layers: &'a [PlannedLayer],
    pub(crate) layer_slices: &'a [ExPolygon],
    pub(crate) object: &'a ObjectOptions,
    pub(crate) default_region: &'a ProcessRegionSourceOptions,
    pub(crate) nozzles: &'a OrcaFloats,
    pub(crate) scale: CoordinateScale,
}

pub(crate) fn append_spans(input: SpanGeneration<'_>) -> Result<(), SliceError> {
    let region_count = input.prelude.records.len() / input.layers.len();
    let mut spans = Vec::with_capacity(region_count);
    for region_index in 0..region_count {
        let record_index = region_index * input.layers.len() + input.layer_index;
        if let (Some(record), Some(record_input)) = (
            input.prepared.predecessor.objects[input.object_index].records[record_index].as_ref(),
            input.prelude.records[record_index].as_ref(),
        ) {
            let region = input.prelude.region_options(record_input);
            let model_rotation_offset = if region.align_infill_direction_to_model.0 {
                record_input.model_rotation_rad as f32
            } else {
                0.0
            };
            let context = SpanContext {
                object: input.object,
                default_region: input.default_region,
                layer: input.layer,
                layers: input.layers,
                model_rotation_offset,
            };
            spans.push((span_sort_key(region, &context), record, record_input));
        }
    }
    spans.sort_by(|(left, _, _), (right, _, _)| left.cmp(right));
    let mut previous_ironing_pattern = None;
    for (_, record, record_input) in spans {
        let region = input.prelude.region_options(record_input);
        let collection_start = input.output.collections.len();
        let pattern_replaced =
            previous_ironing_pattern.is_some_and(|previous| previous != region.ironing_pattern);
        previous_ironing_pattern = Some(region.ironing_pattern);
        append(
            input.output,
            Input {
                record,
                layer_slices: input.layer_slices,
                region,
                dont_alternate_fill_direction: region.zaa_enabled.0
                    && region.zaa_dont_alternate_fill_direction.0
                    && !pattern_replaced,
                object: input.object,
                default_region: input.default_region,
                nozzles: input.nozzles,
                layer: input.layer,
                layers: input.layers,
                last_layer: record_input.upper_layer_index.is_none(),
                spiral_mode: record_input.spiral_mode,
                model_rotation_offset: if region.align_infill_direction_to_model.0 {
                    record_input.model_rotation_rad as f32
                } else {
                    0.0
                },
                scale: input.scale,
            },
        )?;
        input.output.collection_ranges.push((
            record_input.current.region_index,
            collection_start..input.output.collections.len(),
        ));
    }
    Ok(())
}
