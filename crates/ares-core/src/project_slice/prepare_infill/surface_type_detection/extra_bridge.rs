use crate::{
    ProcessExtraBridgeLayer, SliceError,
    geometry::{
        CoordinateScale, ExPolygon, JoinType, difference_ex_with_safety_offset,
        intersection_ex_with_safety_offset, offset_expolygons, opening_ex,
        union_safety_offset_expolygons,
    },
    project_slice::{
        perimeters::types::{PerimeterInputRecord, PostPerimeterInputPrintObject},
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

use super::{geometry::geometry_error, types::StagedRecord};

struct Cache {
    polygons: Vec<ExPolygon>,
    offset: f32,
}

pub(super) fn apply(
    records: &mut [Option<StagedRecord>],
    inputs: &[Option<PerimeterInputRecord>],
    input_object: &PostPerimeterInputPrintObject,
    mode: ProcessExtraBridgeLayer,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    if !matches!(
        mode,
        ProcessExtraBridgeLayer::ExternalBridgeOnly | ProcessExtraBridgeLayer::ApplyToAll
    ) {
        return Ok(());
    }
    let caches = collect(records, inputs, input_object, scale)?;
    for (cache, input) in caches.iter().zip(inputs) {
        let (Some(cache), Some(input)) = (cache, input) else {
            continue;
        };
        let Some(upper_same_region) = input.upper_same_region else {
            continue;
        };
        let upper_index = input_object.region_layer_record_index(upper_same_region);
        let Some(upper) = &mut records[upper_index] else {
            continue;
        };
        upper.slices = rewrite(std::mem::take(&mut upper.slices), cache)?;
    }
    Ok(())
}

fn collect(
    records: &[Option<StagedRecord>],
    inputs: &[Option<PerimeterInputRecord>],
    input_object: &PostPerimeterInputPrintObject,
    scale: CoordinateScale,
) -> Result<Vec<Option<Cache>>, SliceError> {
    records
        .iter()
        .zip(inputs)
        .map(|(record, input)| match (record, input) {
            (Some(record), Some(input)) => {
                let polygons = record
                    .slices
                    .iter()
                    .filter(|surface| surface.as_parts().0 == RegionSurfaceKind::BottomBridge)
                    .map(|surface| surface.as_parts().1.clone())
                    .collect::<Vec<_>>();
                if polygons.is_empty() {
                    return Ok(None);
                }
                let internal_walls = input_object.region_options(input).wall_loops.0.max(1) - 1;
                let external = scale
                    .checked_scale(f64::from(input.ext_perimeter_flow.width))
                    .expect("validated external flow width is in range");
                let internal = scale
                    .checked_scale(f64::from(input.perimeter_flow.width))
                    .expect("validated perimeter flow width is in range");
                let offset = (external + internal * i64::from(internal_walls)) as f32;
                Ok(Some(Cache { polygons, offset }))
            }
            (None, None) => Ok(None),
            _ => unreachable!("external extra-bridge records remain aligned"),
        })
        .collect()
}

fn rewrite(surfaces: Vec<RegionSurface>, cache: &Cache) -> Result<Vec<RegionSurface>, SliceError> {
    let top = surfaces
        .iter()
        .filter(|surface| surface.as_parts().0 == RegionSurfaceKind::Top)
        .map(|surface| surface.as_parts().1.clone())
        .collect::<Vec<_>>();
    let top = if top.is_empty() {
        Vec::new()
    } else {
        offset_expolygons(&top, cache.offset, JoinType::Miter, 3.0).map_err(geometry_error)?
    };
    let mut output = Vec::new();
    for surface in surfaces {
        let (kind, expolygon, _, _, _, _) = surface.as_parts();
        if kind != RegionSurfaceKind::Internal {
            output.push(surface);
            continue;
        }
        let mut overlap =
            intersection_ex_with_safety_offset(std::slice::from_ref(expolygon), &cache.polygons)
                .and_then(|overlap| opening_ex(&overlap, cache.offset, JoinType::Miter, 3.0))
                .map_err(geometry_error)?;
        if !top.is_empty() && !overlap.is_empty() {
            overlap = difference_ex_with_safety_offset(&overlap, &top).map_err(geometry_error)?;
        }
        let remainder = difference_ex_with_safety_offset(std::slice::from_ref(expolygon), &overlap)
            .and_then(|remainder| union_safety_offset_expolygons(&remainder))
            .map_err(geometry_error)?;
        output.extend(
            remainder
                .into_iter()
                .map(|expolygon| surface.clone_with_expolygon(expolygon)),
        );
        let overlap = union_safety_offset_expolygons(&overlap).map_err(geometry_error)?;
        output.extend(overlap.into_iter().map(|expolygon| {
            let mut bridge = surface.clone_with_expolygon(expolygon);
            bridge.retag(RegionSurfaceKind::BottomBridge);
            bridge
        }));
    }
    Ok(output)
}
