//! `PrintObject::combine_infill` (`PrintObject.cpp:4166-4291`).

use crate::{
    FloatOrPercent, OrcaFloats, ProcessInfillPattern, RegionOptions, SliceError,
    geometry::{
        CoordinateScale, ExPolygon, JoinType, difference_ex, intersection_ex, offset_expolygons,
    },
    project_slice::{
        perimeters::{
            classic::traversal::PostClassicTraversalPrintObject,
            types::{Flow, PerimeterFlows},
        },
        prepare_infill::surface_type_detection::{
            PreparedSurfaceTypeObject, types::PreparedSurfaceTypeRecord,
        },
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

const EPSILON: f64 = 1.0e-4;

#[derive(Clone)]
struct LayerConfig {
    active: bool,
    kind: RegionSurfaceKind,
    pattern: ProcessInfillPattern,
    maximum_height: f64,
    area_threshold: f64,
    layer_id: usize,
    layer_height: f64,
    flows: PerimeterFlows,
}

struct CombinedGeometry<'a> {
    kind: RegionSurfaceKind,
    intersection: &'a [ExPolygon],
    clearance: &'a [ExPolygon],
    thickness: f64,
    layers: u16,
}

pub(super) fn apply(
    object: &mut PreparedSurfaceTypeObject,
    traversal: &PostClassicTraversalPrintObject,
    nozzles: &OrcaFloats,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    let prelude = &traversal.predecessor.predecessor.predecessor.predecessor;
    let (post_regions, _) = prelude.object.object.as_parts();
    let (plan, _, regions) = post_regions.as_parts();
    if regions.iter().all(|region| {
        let options = region.as_parts().1;
        !options.infill_combination.0 || options.sparse_infill_density.0 == 0.0
    }) {
        return Ok(());
    }
    let configs = layer_configs(traversal, nozzles, scale)?;
    for region_index in 0..regions.len() {
        let start = region_index * plan.layers.len();
        let end = start + plan.layers.len();
        let configs = &configs[start..end];
        let records = &mut object.records[start..end];
        let groups = groups(configs);
        for (top, count) in groups
            .into_iter()
            .enumerate()
            .filter(|(_, count)| *count > 1)
        {
            combine_group(records, configs, top, count, scale)?;
        }
    }
    Ok(())
}

fn layer_configs(
    traversal: &PostClassicTraversalPrintObject,
    nozzles: &OrcaFloats,
    scale: CoordinateScale,
) -> Result<Vec<LayerConfig>, SliceError> {
    let prelude = &traversal.predecessor.predecessor.predecessor.predecessor;
    prelude
        .object
        .records
        .iter()
        .map(|input| {
            let input = input.as_ref().expect("combined fill has an aligned input");
            let options = prelude.object.region_options(input);
            let kind = if (options.sparse_infill_density.0 - 100.0).abs() < EPSILON {
                RegionSurfaceKind::InternalSolid
            } else {
                RegionSurfaceKind::Internal
            };
            let pattern = if kind == RegionSurfaceKind::InternalSolid {
                options.internal_solid_infill_pattern
            } else {
                options.sparse_infill_pattern
            };
            let nozzle = selected_nozzle(nozzles, options.sparse_infill_filament_id.0).min(
                selected_nozzle(nozzles, options.internal_solid_filament_id.0),
            );
            let configured = absolute_height(options, nozzle);
            let maximum_height = if configured > 0.0 {
                configured.min(nozzle)
            } else {
                nozzle
            };
            Ok(LayerConfig {
                active: options.infill_combination.0 && options.sparse_infill_density.0 != 0.0,
                kind,
                pattern,
                maximum_height,
                area_threshold: infill_area_threshold(input.solid_infill_flow, scale),
                layer_id: input.layer_id,
                layer_height: input.layer_height,
                flows: PerimeterFlows {
                    perimeter_flow: input.perimeter_flow,
                    ext_perimeter_flow: input.ext_perimeter_flow,
                    overhang_flow: input.overhang_flow,
                    solid_infill_flow: input.solid_infill_flow,
                },
            })
        })
        .collect()
}

fn selected_nozzle(nozzles: &OrcaFloats, selector: i32) -> f64 {
    let index = selector
        .checked_sub(1)
        .and_then(|index| usize::try_from(index).ok())
        .filter(|index| *index < nozzles.0.len())
        .unwrap_or(0);
    nozzles.0[index].0
}

fn absolute_height(options: &RegionOptions, nozzle: f64) -> f64 {
    match options.infill_combination_max_layer_height {
        FloatOrPercent::Float(value) => value,
        FloatOrPercent::Percent(value) => value.0 * 0.01 * nozzle,
    }
}

/// `LayerRegion::infill_area_threshold` (`LayerRegion.cpp:975-979`):
/// `flow(frSolidInfill).scaled_spacing()` squared.
fn infill_area_threshold(solid_infill_flow: Flow, scale: CoordinateScale) -> f64 {
    let spacing = (f64::from(solid_infill_flow.spacing) / scale.factor()) as i64 as f64;
    spacing * spacing
}

fn groups(configs: &[LayerConfig]) -> Vec<usize> {
    let mut output = vec![0; configs.len()];
    let mut current_height = 0.0;
    let mut count = 0;
    for (index, config) in configs.iter().enumerate() {
        if config.layer_id == 0 {
            continue;
        }
        if !config.active {
            current_height = 0.0;
            count = 0;
            continue;
        }
        if current_height + config.layer_height >= config.maximum_height + EPSILON {
            output[index - 1] = count;
            current_height = 0.0;
            count = 0;
        }
        current_height += config.layer_height;
        count += 1;
    }
    if let Some(last) = output.last_mut() {
        *last = count;
    }
    output
}

fn combine_group(
    records: &mut [Option<PreparedSurfaceTypeRecord>],
    configs: &[LayerConfig],
    top: usize,
    count: usize,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    let bottom = top + 1 - count;
    let kind = configs[top].kind;
    let mut intersection = surfaces_of_kind(record(records, bottom), kind);
    for index in bottom + 1..=top {
        intersection = intersection_ex(
            &surfaces_of_kind(record(records, index), kind),
            &intersection,
        )
        .map_err(geometry_error)?;
    }
    let threshold = configs[bottom].area_threshold;
    if threshold > 0.0 {
        intersection.retain(|expolygon| expolygon.area() > threshold);
    }
    if intersection.is_empty() {
        return Ok(());
    }
    let factor = if clearance_pattern(configs[top].pattern) {
        1.5
    } else {
        0.5
    };
    let clearance = 0.5_f32 * configs[top].flows.perimeter_flow.scaled_width(scale) as f32
        + factor * configs[top].flows.solid_infill_flow.scaled_width(scale) as f32;
    let clearance = offset_expolygons(&intersection, clearance, JoinType::Miter, 3.0)
        .map_err(geometry_error)?;
    let thickness = configs[bottom..=top]
        .iter()
        .map(|config| config.layer_height)
        .sum::<f64>();
    let geometry = CombinedGeometry {
        kind,
        intersection: &intersection,
        clearance: &clearance,
        thickness,
        layers: u16::try_from(count).unwrap_or(u16::MAX),
    };
    for index in bottom..=top {
        rewrite_layer(record_mut(records, index), &geometry, index == top)?;
    }
    Ok(())
}

fn rewrite_layer(
    record: &mut PreparedSurfaceTypeRecord,
    geometry: &CombinedGeometry<'_>,
    top: bool,
) -> Result<(), SliceError> {
    let mut retained = Vec::new();
    let mut internal = Vec::new();
    for surface in std::mem::take(&mut record.fill_surfaces) {
        if surface.as_parts().0 == geometry.kind {
            internal.push(surface.into_parts().1);
        } else {
            retained.push(surface);
        }
    }
    retained.extend(
        difference_ex(&internal, geometry.clearance)
            .map_err(geometry_error)?
            .into_iter()
            .map(|expolygon| RegionSurface::new(geometry.kind, expolygon)),
    );
    if top {
        retained.extend(geometry.intersection.iter().cloned().map(|expolygon| {
            let mut surface = RegionSurface::new(geometry.kind, expolygon);
            surface.set_thickness(geometry.thickness);
            surface.set_thickness_layers(geometry.layers);
            surface
        }));
    } else {
        retained.extend(
            intersection_ex(&internal, geometry.clearance)
                .map_err(geometry_error)?
                .into_iter()
                .map(|expolygon| RegionSurface::new(RegionSurfaceKind::InternalVoid, expolygon)),
        );
    }
    record.fill_surfaces = retained;
    Ok(())
}

fn surfaces_of_kind(record: &PreparedSurfaceTypeRecord, kind: RegionSurfaceKind) -> Vec<ExPolygon> {
    record
        .fill_surfaces
        .iter()
        .filter(|surface| surface.as_parts().0 == kind)
        .map(|surface| surface.as_parts().1.clone())
        .collect()
}

fn record(
    records: &[Option<PreparedSurfaceTypeRecord>],
    index: usize,
) -> &PreparedSurfaceTypeRecord {
    records[index]
        .as_ref()
        .expect("combined layer has a fill record")
}

fn record_mut(
    records: &mut [Option<PreparedSurfaceTypeRecord>],
    index: usize,
) -> &mut PreparedSurfaceTypeRecord {
    records[index]
        .as_mut()
        .expect("combined layer has a fill record")
}

fn clearance_pattern(pattern: ProcessInfillPattern) -> bool {
    matches!(
        pattern,
        ProcessInfillPattern::Rectilinear
            | ProcessInfillPattern::Monotonic
            | ProcessInfillPattern::Grid
            | ProcessInfillPattern::LateralLattice
            | ProcessInfillPattern::Line
            | ProcessInfillPattern::Honeycomb
            | ProcessInfillPattern::LateralHoneycomb
    )
}

fn geometry_error(error: crate::geometry::ClipperError) -> SliceError {
    match error {
        crate::geometry::ClipperError::CoordinateOutOfRange => SliceError::InvalidInput(
            "combined-infill coordinate is outside the supported Clipper range".to_owned(),
        ),
    }
}
