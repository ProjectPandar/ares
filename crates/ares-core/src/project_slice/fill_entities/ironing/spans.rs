use crate::{ObjectOptions, OrcaFloats, OrcaInts, ProcessRegionSourceOptions, SliceError};
use crate::{
    geometry::{CoordinateScale, ExPolygon},
    project_slice::{
        layers::PlannedLayer, perimeters::types::PostPerimeterInputPrintObject,
        prepare_infill::external_surfaces::PreparedPostExternalSurfaces,
    },
};

use super::{Input, LayerFillEntities, SpanContext, append_areas, areas, span_sort_key};

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
    pub(crate) filament_map: &'a OrcaInts,
    pub(crate) is_bbl: bool,
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
            let last_layer = record_input.upper_layer_index.is_none();
            let model_rotation_offset = if region.align_infill_direction_to_model.0 {
                record_input.model_rotation_rad
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
            let member = Input {
                record,
                layer_slices: input.layer_slices,
                region,
                dont_alternate_fill_direction: false,
                object: input.object,
                nozzles: input.nozzles,
                filament_map: input.filament_map,
                is_bbl: input.is_bbl,
                layer: input.layer,
                last_layer,
                spiral_mode: record_input.spiral_mode,
                scale: input.scale,
            };
            if record.slices.is_empty()
                || !areas::enabled(
                    region.ironing_type,
                    region,
                    member.last_layer,
                    member.spiral_mode,
                )
            {
                continue;
            }
            spans.push((
                span_sort_key(region, &context),
                member,
                record_input.current.region_index,
            ));
        }
    }
    spans.sort_by(|(left, _, _), (right, _, _)| left.cmp(right));
    let mut previous_ironing_pattern = None;
    let mut i = 0;
    while i < spans.len() {
        let mut j = i + 1;
        while j < spans.len() && spans[i].0 == spans[j].0 {
            j += 1;
        }
        let region = spans[i].1.region;
        let pattern_replaced =
            previous_ironing_pattern.is_some_and(|previous| previous != region.ironing_pattern);
        previous_ironing_pattern = Some(region.ironing_pattern);
        spans[i].1.dont_alternate_fill_direction =
            region.zaa_enabled.0 && region.zaa_dont_alternate_fill_direction.0 && !pattern_replaced;
        let mut polygons = Vec::new();
        let mut infills = Vec::new();
        for (_, member, _) in &spans[i..j] {
            areas::collect(member, &mut polygons, &mut infills);
        }
        let (key, primary, _) = &spans[i];
        let collection_start = input.output.collections.len();
        if let Some(ironing_areas) = areas::ironing_areas(primary, polygons, infills, j > i + 1)? {
            append_areas(
                input.output,
                primary,
                ironing_areas,
                (key.angle, key.fixed_angle),
            )?;
        }
        input
            .output
            .collection_ranges
            .push((spans[i].2, collection_start..input.output.collections.len()));
        i = j;
    }
    Ok(())
}
