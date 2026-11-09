use crate::{
    SliceError,
    geometry::{ExPolygon, intersection_ex},
    project_slice::{
        perimeters::{
            classic::PostClassicTraversalPrintObject,
            flow::{FillFlowContext, FillFlowRole, resolve_fill_flow},
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
    cracks, extra_bridge,
    geometry::{
        GeometryStep, expolygons, fresh, geometry_error, internal, observe, opening,
        opening_offset, paths, safety_difference,
    },
    preflight,
    top_expansion::{self, TopExpansion},
    types::StagedRecord,
};

pub(super) struct StagedObject {
    pub(super) records: Vec<Option<StagedRecord>>,
}

#[derive(Clone, Copy)]
struct StageRecordContext {
    external_width: i64,
    bottom_kind: RegionSurfaceKind,
    bottom_shell_layers: usize,
    layer_count: usize,
    spiral_mode: bool,
    interface_shells: bool,
    top_expansion: Option<TopExpansion>,
}

pub(super) fn project(
    prepared: &PreparedPostLayerRegionPerimeters,
) -> Result<Vec<StagedObject>, SliceError> {
    let classic = prepared.predecessor.as_classic();
    assert_eq!(prepared.objects.len(), classic.objects.len());
    prepared
        .objects
        .iter()
        .zip(&classic.objects)
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
    let source_index = input_object.identity().0;
    let options = &prepared
        .predecessor
        .as_classic()
        .resolved
        .objects
        .iter()
        .find(|resolved| resolved.source_object_index == source_index)
        .expect("O17 object retains its resolved source")
        .object;
    let bottom_kind = preflight::bottom_kind(options);
    let spiral_mode = prepared
        .predecessor
        .as_classic()
        .resolved
        .views
        .full
        .process
        .print
        .spiral_mode
        .0;
    let classic = prepared.predecessor.as_classic();
    let full = &classic.resolved.views.full;
    let (compensated, _) = input_object.as_parts();
    let (post_region, _) = compensated.as_parts();
    let (plan, _, _) = post_region.as_parts();
    let layer_count = input_object
        .records
        .iter()
        .flatten()
        .map(|record| record.layer_id + 1)
        .max()
        .unwrap_or(0);
    let mut records = object
        .records
        .iter()
        .zip(&input_object.records)
        .map(|(output, input)| match (output, input) {
            (Some(output), Some(input)) => {
                let region = input_object.region_options(input);
                let bottom_shell_layers = usize::try_from(region.bottom_shell_layers.0)
                    .expect("normalized bottom_shell_layers is non-negative");
                let top_expansion = TopExpansion::new(
                    region,
                    (input.ext_perimeter_flow, input.perimeter_flow),
                    || {
                        let layer = &plan.layers[input.planned_layer_index];
                        resolve_fill_flow(
                            FillFlowContext::new(
                                layer,
                                layer.height,
                                full.process.print.initial_layer_line_width,
                                region,
                                options,
                                &full.project.print.nozzle_diameter,
                            ),
                            FillFlowRole::Top,
                        )
                    },
                    classic.scale,
                )?;
                stage_record(
                    input_object,
                    input,
                    output,
                    StageRecordContext {
                        external_width: input
                            .ext_perimeter_flow
                            .scaled_width(prepared.predecessor.as_classic().scale),
                        bottom_kind,
                        bottom_shell_layers,
                        layer_count,
                        spiral_mode,
                        interface_shells: options.interface_shells.0,
                        top_expansion,
                    },
                )
                .map(Some)
            }
            (None, None) => Ok(None),
            _ => unreachable!("O17 slots remain aligned with O16 and regional inputs"),
        })
        .collect::<Result<Vec<_>, _>>()?;
    extra_bridge::apply(
        &mut records,
        &input_object.records,
        input_object,
        options.enable_extra_bridge_layer,
        prepared.predecessor.as_classic().scale,
    )?;
    for (staged, output) in records.iter_mut().zip(&object.records) {
        match (staged, output) {
            (Some(staged), Some(output)) => {
                staged.fill_surfaces = clipped_fill(&staged.slices, &output.fill_expolygons)?;
            }
            (None, None) => {}
            _ => unreachable!("external extra-bridge records remain aligned"),
        }
    }
    Ok(StagedObject { records })
}

fn stage_record(
    object: &PostPerimeterInputPrintObject,
    input: &PerimeterInputRecord,
    output: &PreparedLayerRegionPerimeterRecord,
    context: StageRecordContext,
) -> Result<StagedRecord, SliceError> {
    let source = object.current_surfaces(input);
    let same_region_upper = context
        .interface_shells
        .then(|| object.upper_same_region_surfaces(input))
        .flatten()
        .map(expolygons);
    let upper = same_region_upper
        .as_deref()
        .or_else(|| object.upper_slices(input));
    let interface_bottom = context
        .interface_shells
        .then(|| object.lower_same_region_surfaces(input))
        .flatten()
        .map(expolygons);
    let lower = object.lower_slices(input).map(|slices| LowerLayer {
        slices,
        own_region: interface_bottom.as_deref(),
    });
    let domain: (&[RegionSurface], Option<&[RegionSurface]>) =
        if object.region_options(input).counterbore_hole_bridging
            == crate::ProcessCounterboreHoleBridging::SacrificialLayer
        {
            (source, Some(output.fill_surfaces.as_slice()))
        } else {
            (source, None)
        };
    let mut slices = classify_with_fill(
        domain,
        upper,
        lower,
        (
            context.external_width,
            context.bottom_kind,
            context.top_expansion,
        ),
    )?;
    apply_spiral_surface_types(
        &mut slices,
        context.spiral_mode,
        input.layer_id,
        context.bottom_shell_layers,
        context.layer_count,
    );
    let fill_surfaces = clipped_fill(&slices, &output.fill_expolygons)?;
    Ok(StagedRecord {
        slices,
        fill_surfaces,
    })
}

pub(super) fn apply_spiral_surface_types(
    surfaces: &mut [RegionSurface],
    spiral_mode: bool,
    layer_id: usize,
    bottom_shell_layers: usize,
    layer_count: usize,
) {
    if !spiral_mode {
        return;
    }
    let base_layer_count = bottom_shell_layers.min(layer_count);
    let kind = if base_layer_count > 1 && layer_id + 1 == base_layer_count {
        Some(RegionSurfaceKind::Top)
    } else if layer_id >= base_layer_count {
        Some(RegionSurfaceKind::Internal)
    } else {
        None
    };
    if let Some(kind) = kind {
        for surface in surfaces {
            surface.retag(kind);
        }
    }
}

/// The lower neighbor layer: its whole slices plus, when `interface_shells` is
/// set, this region's own slices on that layer.
#[derive(Clone, Copy)]
pub(super) struct LowerLayer<'a> {
    pub(super) slices: &'a [ExPolygon],
    pub(super) own_region: Option<&'a [ExPolygon]>,
}

pub(super) fn classify_with_fill(
    domain: (&[RegionSurface], Option<&[RegionSurface]>),
    upper: Option<&[ExPolygon]>,
    lower: Option<LowerLayer<'_>>,
    (external_width, bottom_kind, expansion): (i64, RegionSurfaceKind, Option<TopExpansion>),
) -> Result<Vec<RegionSurface>, SliceError> {
    let (source, generated) = domain;
    let previous = expolygons(source);
    // PrintObject.cpp:1579–1583 expands comparisons, not the original internal domain (:1698).
    let filled_comparison = generated
        .map(|fill| crate::geometry::union_expolygons(&previous, &expolygons(fill)))
        .transpose()
        .map_err(geometry_error)?;
    let comparison = filled_comparison.as_deref().unwrap_or(&previous);
    let offset = opening_offset(external_width);

    let mut top = if let Some(upper) = upper {
        fresh(
            RegionSurfaceKind::Top,
            opening(
                &safety_difference(comparison, upper, GeometryStep::TopSafetyDifference)?,
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
        let mut surfaces = fresh(
            bottom_kind,
            opening(
                &safety_difference(
                    comparison,
                    lower.slices,
                    GeometryStep::BottomSafetyDifference,
                )?,
                offset,
                GeometryStep::BottomShrink,
                GeometryStep::BottomExpand,
            )?,
        );
        surfaces.extend(interface_bottom_surfaces(comparison, &lower, offset)?);
        surfaces
    } else {
        source
            .iter()
            .map(|surface| surface.clone_with_kind(RegionSurfaceKind::Bottom))
            .collect()
    };

    cracks::resolve(&mut top, &mut bottom, external_width, lower.is_some())?;
    if let Some(expansion) = expansion {
        top_expansion::apply(expansion, &mut top, &bottom, comparison)?;
    }
    let mut top_bottom = paths(&top);
    top_bottom.extend(paths(&bottom));
    let mut slices = internal(&previous, &top_bottom)?;
    slices.extend(top);
    slices.extend(bottom);
    Ok(slices)
}

/// Ported from PrintObject.cpp (interface_shells `#else` path): the portion of
/// this layer supported by a different region below becomes a solid bottom.
fn interface_bottom_surfaces(
    comparison: &[ExPolygon],
    lower: &LowerLayer<'_>,
    offset: f32,
) -> Result<Vec<RegionSurface>, SliceError> {
    let Some(own_lower) = lower.own_region else {
        return Ok(Vec::new());
    };
    let supported = intersection_ex(comparison, lower.slices).map_err(geometry_error)?;
    let other_region = safety_difference(
        &supported,
        own_lower,
        GeometryStep::InterfaceBottomDifference,
    )?;
    Ok(fresh(
        RegionSurfaceKind::Bottom,
        opening(
            &other_region,
            offset,
            GeometryStep::BottomShrink,
            GeometryStep::BottomExpand,
        )?,
    ))
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
