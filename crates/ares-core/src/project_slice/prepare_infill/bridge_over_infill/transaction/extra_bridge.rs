#[cfg(test)]
mod tests;

use crate::{
    ProcessExtraBridgeLayer, SliceError,
    geometry::{
        CoordinateScale, ExPolygon, JoinType, difference_ex_with_safety_offset,
        intersection_ex_with_safety_offset, opening_ex, union_expolygons,
        union_safety_offset_expolygons,
    },
    project_slice::{
        perimeters::types::PerimeterInputRecord,
        prepare_infill::{
            bridge_over_infill::transaction::geometry_error,
            external_surfaces::PreparedPostExternalSurfaces,
            surface_type_detection::types::PreparedSurfaceTypeRecord,
        },
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

struct Cache {
    polygons: Vec<ExPolygon>,
    angle: f64,
    offset: f32,
}

pub(super) fn prepare(predecessor: &mut PreparedPostExternalSurfaces) -> Result<(), SliceError> {
    let horizontal = &mut predecessor.predecessor;
    let traversal = &horizontal.predecessor;
    let scale = traversal.scale;
    for object_index in 0..horizontal.objects.len() {
        if !matches!(
            traversal.resolved.objects[object_index]
                .object
                .enable_extra_bridge_layer,
            ProcessExtraBridgeLayer::InternalBridgeOnly | ProcessExtraBridgeLayer::ApplyToAll
        ) {
            continue;
        }
        let prelude = &traversal.objects[object_index]
            .predecessor
            .predecessor
            .predecessor
            .predecessor;
        let (_, inputs) = prelude.object.as_parts();
        let caches = collect(&horizontal.objects[object_index].records, inputs, scale)?;
        apply(&mut horizontal.objects[object_index].records, &caches)?;
    }
    Ok(())
}

fn collect(
    records: &[Option<PreparedSurfaceTypeRecord>],
    inputs: &[Option<PerimeterInputRecord>],
    scale: CoordinateScale,
) -> Result<Vec<Option<Cache>>, SliceError> {
    records
        .iter()
        .zip(inputs)
        .map(|(record, input)| match (record, input) {
            (Some(record), Some(input)) => {
                let mut polygons = Vec::new();
                let mut angle = 0.0;
                for surface in &record.fill_surfaces {
                    let (kind, expolygon, _, _, bridge_angle, _) = surface.as_parts();
                    if kind == RegionSurfaceKind::InternalBridge {
                        polygons.push(expolygon.clone());
                        angle = bridge_angle;
                    }
                }
                if polygons.is_empty() {
                    return Ok(None);
                }
                let offset = scale
                    .checked_scale(f64::from(input.solid_infill_flow.width))
                    .expect("validated solid-infill width is in the coordinate range")
                    as f32;
                let polygons =
                    opening_ex(&polygons, offset, JoinType::Miter, 3.0).map_err(geometry_error)?;
                Ok((!polygons.is_empty()).then_some(Cache {
                    polygons,
                    angle,
                    offset,
                }))
            }
            (None, None) => Ok(None),
            _ => unreachable!("extra-bridge cache records remain aligned"),
        })
        .collect()
}

fn apply(
    records: &mut [Option<PreparedSurfaceTypeRecord>],
    caches: &[Option<Cache>],
) -> Result<(), SliceError> {
    for layer_index in 0..records.len().saturating_sub(1) {
        let Some(cache) = &caches[layer_index] else {
            continue;
        };
        let Some(upper) = &mut records[layer_index + 1] else {
            continue;
        };
        let union = union_safety_offset_expolygons(&cache.polygons).map_err(geometry_error)?;
        upper.fill_surfaces =
            rewrite_surfaces(std::mem::take(&mut upper.fill_surfaces), &union, cache)?;
    }
    Ok(())
}

fn rewrite_surfaces(
    surfaces: Vec<RegionSurface>,
    bridge_union: &[ExPolygon],
    cache: &Cache,
) -> Result<Vec<RegionSurface>, SliceError> {
    let mut output = Vec::new();
    for surface in surfaces {
        let (kind, expolygon, _, _, _, _) = surface.as_parts();
        if !matches!(
            kind,
            RegionSurfaceKind::Internal | RegionSurfaceKind::InternalSolid
        ) {
            output.push(surface);
            continue;
        }
        let overlap =
            intersection_ex_with_safety_offset(std::slice::from_ref(expolygon), bridge_union)
                .and_then(|overlap| opening_ex(&overlap, cache.offset, JoinType::Miter, 3.0))
                .map_err(geometry_error)?;
        if overlap.is_empty() {
            output.push(surface);
            continue;
        }
        output.extend(overlap.into_iter().map(|expolygon| {
            let mut bridge = surface.clone_with_expolygon(expolygon);
            bridge.retag(RegionSurfaceKind::InternalBridge);
            bridge.set_bridge_angle(cache.angle + std::f64::consts::FRAC_PI_2);
            bridge
        }));
        let leftover =
            difference_ex_with_safety_offset(std::slice::from_ref(expolygon), bridge_union)
                .and_then(|leftover| opening_ex(&leftover, cache.offset, JoinType::Miter, 3.0))
                .and_then(|leftover| union_expolygons(&leftover, &[]))
                .map_err(geometry_error)?;
        output.extend(
            leftover
                .into_iter()
                .map(|expolygon| surface.clone_with_expolygon(expolygon)),
        );
    }
    Ok(output)
}
