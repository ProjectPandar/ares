use crate::{
    SliceError,
    geometry::{ExPolygon, intersection_ex},
    project_slice::{
        perimeters::{
            classic::PostClassicTraversalPrintObject,
            layer_region::{
                PreparedLayerRegionPerimeterObject, PreparedLayerRegionPerimeterRecord,
                PreparedPostLayerRegionPerimeters,
            },
            types::{PerimeterInputRecord, PostPerimeterInputPrintObject},
        },
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

use super::{
    cracks,
    geometry::{
        GeometryStep, expolygons, fresh, geometry_error, internal, observe, opening,
        opening_offset, paths, safety_difference,
    },
    preflight,
    types::StagedRecord,
};

pub(super) struct StagedObject {
    pub(super) records: Vec<Option<StagedRecord>>,
}

pub(super) fn project(
    prepared: &PreparedPostLayerRegionPerimeters,
) -> Result<Vec<StagedObject>, SliceError> {
    assert_eq!(prepared.objects.len(), prepared.predecessor.objects.len());
    prepared
        .objects
        .iter()
        .zip(&prepared.predecessor.objects)
        .map(|(object, traversal)| stage_object(prepared, object, traversal))
        .collect()
}

fn stage_object(
    prepared: &PreparedPostLayerRegionPerimeters,
    object: &PreparedLayerRegionPerimeterObject,
    traversal: &PostClassicTraversalPrintObject,
) -> Result<StagedObject, SliceError> {
    let prelude = &traversal.predecessor.predecessor.predecessor.predecessor;
    let input_object = &prelude.object;
    assert_eq!(object.records.len(), input_object.records.len());
    assert_eq!(object.records.len(), prelude.records.len());
    let source_index = input_object.identity().0;
    let options = &prepared
        .predecessor
        .resolved
        .objects
        .iter()
        .find(|resolved| resolved.source_object_index == source_index)
        .expect("O17 object retains its resolved source")
        .object;
    let bottom_kind = preflight::bottom_kind(options);
    let records = object
        .records
        .iter()
        .zip(&input_object.records)
        .zip(&prelude.records)
        .map(
            |((output, input), prelude)| match (output, input, prelude) {
                (Some(output), Some(input), Some(prelude)) => stage_record(
                    input_object,
                    input,
                    prelude.external_width,
                    bottom_kind,
                    output,
                )
                .map(Some),
                (None, None, None) => Ok(None),
                _ => unreachable!("O17 slots remain aligned with O16 and the Classic prelude"),
            },
        )
        .collect::<Result<Vec<_>, _>>()?;
    Ok(StagedObject { records })
}

fn stage_record(
    object: &PostPerimeterInputPrintObject,
    input: &PerimeterInputRecord,
    external_width: i64,
    bottom_kind: RegionSurfaceKind,
    output: &PreparedLayerRegionPerimeterRecord,
) -> Result<StagedRecord, SliceError> {
    let source = object.current_surfaces(input);
    let slices = classify_slices(
        source,
        object.upper_slices(input),
        object.lower_slices(input),
        external_width,
        bottom_kind,
    )?;
    let fill_surfaces = clipped_fill(&slices, &output.fill_expolygons)?;
    Ok(StagedRecord {
        slices,
        fill_surfaces,
    })
}

pub(super) fn classify_slices(
    source: &[RegionSurface],
    upper: Option<&[ExPolygon]>,
    lower: Option<&[ExPolygon]>,
    external_width: i64,
    bottom_kind: RegionSurfaceKind,
) -> Result<Vec<RegionSurface>, SliceError> {
    let previous = expolygons(source);
    let offset = opening_offset(external_width);

    let mut top = if let Some(upper) = upper {
        fresh(
            RegionSurfaceKind::Top,
            opening(
                &safety_difference(&previous, upper, GeometryStep::TopSafetyDifference)?,
                offset,
                GeometryStep::TopShrink,
                GeometryStep::TopExpand,
            )?,
        )
    } else {
        source
            .iter()
            .map(|surface| surface.clone_with_kind(RegionSurfaceKind::Top))
            .collect()
    };
    let mut bottom = if let Some(lower) = lower {
        fresh(
            bottom_kind,
            opening(
                &safety_difference(&previous, lower, GeometryStep::BottomSafetyDifference)?,
                offset,
                GeometryStep::BottomShrink,
                GeometryStep::BottomExpand,
            )?,
        )
    } else {
        source
            .iter()
            .map(|surface| surface.clone_with_kind(RegionSurfaceKind::Bottom))
            .collect()
    };

    cracks::resolve(&mut top, &mut bottom, external_width, lower.is_some())?;
    let mut top_bottom = paths(&top);
    top_bottom.extend(paths(&bottom));
    let mut slices = internal(&previous, &top_bottom)?;
    slices.extend(top);
    slices.extend(bottom);
    Ok(slices)
}

pub(super) fn clipped_fill(
    slices: &[RegionSurface],
    boundaries: &[ExPolygon],
) -> Result<Vec<RegionSurface>, SliceError> {
    let mut output = Vec::new();
    for (kind, step) in [
        (RegionSurfaceKind::Top, GeometryStep::FillTopIntersection),
        (
            RegionSurfaceKind::Bottom,
            GeometryStep::FillBottomIntersection,
        ),
        (
            RegionSurfaceKind::BottomBridge,
            GeometryStep::FillBottomBridgeIntersection,
        ),
        (
            RegionSurfaceKind::Internal,
            GeometryStep::FillInternalIntersection,
        ),
    ] {
        let group = slices
            .iter()
            .filter(|surface| surface.as_parts().0 == kind)
            .map(|surface| surface.as_parts().1.clone())
            .collect::<Vec<_>>();
        if !group.is_empty() {
            observe(step)?;
            let clipped = intersection_ex(&group, boundaries).map_err(geometry_error)?;
            output.extend(fresh(kind, clipped));
        }
    }
    Ok(output)
}
