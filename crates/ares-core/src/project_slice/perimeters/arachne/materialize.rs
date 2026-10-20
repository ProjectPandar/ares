// `PerimeterGenerator::process_arachne` materialization from OrcaSlicer
// v2.4.2 `PerimeterGenerator.cpp:2093-2521`: the per-record transaction that
// generates arachne walls, orders them, converts them through
// `traverse_extrusions` (`:370-574`) into the typed entity vocabulary and
// produces the layer-region perimeter record. Classic records are untouched;
// arachne records carry no classic surfaces, so the classic chain computes
// their shared context only and this stage replaces their entities at the
// layer-region boundary.

use crate::{
    SliceError,
    geometry::difference_ex_polygons,
    project_slice::perimeters::{
        classic::{ClassicPreludeRecord, PreparedPostClassicPrelude, extra_perimeters},
        layer_region::PreparedLayerRegionPerimeterRecord,
        types::{PerimeterDispatch, PerimeterInputRecord, PostPerimeterInputPrintObject},
    },
};

use super::{
    config::{self},
    order::order_walls,
    traverse::{self, TraverseExtrusionsContext},
    walls::{self, ArachneSpacings, InfillOverlapPercents},
};

/// The arachne half of `PreparedPostLayerRegionPerimeters`: one record slot
/// per classic record, `Some` exactly where the record dispatches arachne.
pub(in crate::project_slice) struct PreparedArachnePerimeters {
    pub(in crate::project_slice) objects: Vec<PreparedArachneObject>,
}

#[derive(Default)]
pub(in crate::project_slice) struct PreparedArachneObject {
    pub(in crate::project_slice) records: Vec<Option<PreparedLayerRegionPerimeterRecord>>,
}

pub(in crate::project_slice) fn finish(
    prelude: &PreparedPostClassicPrelude,
    context: &mut crate::SlicerContext,
) -> Result<PreparedArachnePerimeters, SliceError> {
    let enable_arc_fitting = prelude
        .resolved
        .views
        .full
        .process
        .gcode
        .enable_arc_fitting
        .0;
    let resolution = prelude.resolved.views.full.process.print.resolution.0;
    let nozzle_diameters = &prelude.resolved.views.full.project.print.nozzle_diameter;
    let objects = prelude
        .objects
        .iter()
        .map(|object| {
            let (source_object_index, _) = object.object.identity();
            let object_options = &prelude
                .resolved
                .objects
                .iter()
                .find(|resolved| resolved.source_object_index == source_object_index)
                .expect("the prelude object retains its resolved source")
                .object;
            let inputs = object.object.as_parts().1;
            let mut records = (0..inputs.len()).map(|_| None).collect::<Vec<_>>();
            // `Layer.cpp::make_perimeters` runs physical-layer compatible groups
            // in discovery order. Keep that RNG draw order while storing output
            // in the existing region-major record slots.
            let mut order = (0..inputs.len()).collect::<Vec<_>>();
            order.sort_by_key(|&slot| object.object.record_groups[slot]);
            for slot in order {
                let (Some(input), Some(prelude_record)) = (inputs[slot], &object.records[slot])
                else {
                    continue;
                };
                if input.dispatch != PerimeterDispatch::Arachne {
                    continue;
                }
                records[slot] = Some(materialize_record(
                    &object.object,
                    (&input, prelude_record),
                    object_options,
                    config::PrintParameters {
                        enable_arc_fitting,
                        resolution,
                        nozzle_diameters,
                        scale: prelude.scale,
                    },
                    context,
                )?);
            }
            Ok(PreparedArachneObject { records })
        })
        .collect::<Result<Vec<_>, SliceError>>()?;
    Ok(PreparedArachnePerimeters { objects })
}

fn materialize_record(
    object: &PostPerimeterInputPrintObject,
    (input, prelude_record): (&PerimeterInputRecord, &ClassicPreludeRecord),
    object_options: &crate::ObjectOptions,
    config::PrintParameters {
        enable_arc_fitting,
        resolution,
        nozzle_diameters,
        scale,
    }: config::PrintParameters<'_>,
    context: &mut crate::SlicerContext,
) -> Result<PreparedLayerRegionPerimeterRecord, SliceError> {
    let region = object.region_options(input);
    let fuzzy = super::fuzzy::Regions::prepare(object, input, scale)?;
    let config = config::validate_record(
        region,
        object_options,
        input.layer_id,
        input.upper_layer_index,
        crate::project_slice::perimeters::arachne::config::PrintParameters {
            // `process_arachne:2119-2122`: any compatible member can
            // disable the arc-fitting resolution reduction.
            enable_arc_fitting: enable_arc_fitting && !fuzzy.has_contour,
            resolution,
            nozzle_diameters,
            scale,
        },
    )?;
    let (plan, _, _) = object.object.as_parts().0.as_parts();
    let slice_z = plan.layers[input.planned_layer_index].slice_z;
    let spacings = ArachneSpacings::new(
        scale,
        &input.perimeter_flow,
        &input.ext_perimeter_flow,
        &input.solid_infill_flow,
    )?;
    // `min_width_top_surface.get_abs_value(unscale_(perimeter_width))`
    // (`PerimeterGenerator.cpp:2210`): resolve the percent over the
    // perimeter flow width.
    let min_width_top_surface_mm = match region.min_width_top_surface {
        crate::FloatOrPercent::Float(value) => value,
        crate::FloatOrPercent::Percent(percent) => {
            f64::from(input.perimeter_flow.width) * percent.0 / 100.0
        }
    };
    let overlaps = InfillOverlapPercents {
        infill_wall_overlap: region.infill_wall_overlap.0,
        top_bottom_infill_wall_overlap: region.top_bottom_infill_wall_overlap.0,
    };
    // `process_arachne:2131-2134` truncates density to int and adds the
    // alternating wall before per-island extras and single-wall overrides.
    let alternate_extra_wall = region.alternate_extra_wall.0
        && input.layer_id % 2 == 1
        && !input.spiral_mode
        && region.sparse_infill_density.0 as i32 > 0;
    let mut perimeters = Vec::new();
    // `process_no_bridge:1899–1929` precedes ordinary island fill.
    let mut fill_surfaces = prelude_record.counterbore_fill.clone();
    let mut fill_no_overlap = Vec::new();
    let geometry_error =
        |error| SliceError::InvalidInput(format!("extra overhang perimeter geometry: {error:?}"));
    let extra = (!input.spiral_mode
        && object.lower_slices(input).is_some()
        && region.detect_overhang_wall.0
        && region.extra_perimeters_on_overhangs.0
        && region.wall_loops.0 > 0
        && i64::try_from(input.layer_id).unwrap() > i64::from(config.raft_layers))
    .then_some(extra_perimeters::Context {
        lower: &prelude_record.lower_slices_polygons,
        perimeter_count: region.wall_loops.0,
        flow: input.overhang_flow,
        scale,
    });
    // we need each island separately (`PerimeterGenerator.cpp:2124-2126`).
    for surface in object.perimeter_surfaces(input) {
        // `only_one_wall_top` regeneration inputs (`PerimeterGenerator.cpp:2160-2175`):
        // gated on the upper layer existing (non-topmost).
        let top_surface =
            object
                .upper_slices(input)
                .map(|upper_slices| super::top_surface::TopSurfaceInputs {
                    upper_slices,
                    upper_same_region: object.upper_same_region_surfaces(input),
                    lower_slices: object.lower_slices(input),
                    interface_shells: object_options.interface_shells.0,
                    min_width_top_surface_mm,
                    perimeter_width: spacings.perimeter_width,
                });
        let generated = walls::generate_surface_walls(
            surface,
            &config,
            &spacings,
            crate::project_slice::perimeters::arachne::walls::SurfaceWallParameters {
                wall_loops: region.wall_loops.0 + i32::from(alternate_extra_wall),
                layer_height: input.layer_height,
                scale,
            },
            top_surface.as_ref(),
        )?;
        // `loop_number = int(perimeters.size()) - 1`
        // (`PerimeterGenerator.cpp:2466`).
        let loop_number = generated.toolpaths.len() as i32 - 1;
        let ordered = order_walls(&generated.toolpaths, config.outer_wall_first);
        // Inner-outer-inner sandwich reordering (`PerimeterGenerator.cpp:2374-2464`):
        // after the topological sort, gated on layer (skipped on layer 0) and
        // wall count (3 minimum). The proximity thresholds derive from the
        // flow spacings — precise-outer-wall is forced off for this sequence
        // (`PerimeterGenerator.cpp:2409-2417`), so the external threshold is
        // the half-spacing sum.
        let ordered = if region.wall_sequence == crate::ProcessWallSequence::InnerOuterInner
            && input.layer_id > 0
            && ordered.len() > 2
        {
            let threshold_external = (spacings.ext_perimeter_spacing as f64 / 2.0
                + spacings.perimeter_spacing as f64 / 2.0)
                as i64;
            let threshold_internal = spacings.perimeter_spacing;
            super::sandwich::apply_inner_outer_inner(
                ordered,
                threshold_external,
                threshold_internal,
            )
        } else {
            ordered
        };
        let mut outcome = traverse::traverse_extrusions(
            ordered,
            &TraverseExtrusionsContext {
                layer_id: input.layer_id,
                raft_layers: config.raft_layers,
                detect_overhang_wall: region.detect_overhang_wall.0,
                overhang_reverse: region.overhang_reverse.0,
                overhang_reverse_threshold: region.overhang_reverse_threshold,
                wall_direction: region.wall_direction,
                wall_sequence: region.wall_sequence,
                fuzzy_skin: &fuzzy,
                slice_z,
                perimeter_flow: input.perimeter_flow,
                ext_perimeter_flow: input.ext_perimeter_flow,
                overhang_flow: input.overhang_flow,
                lower_slices_polygons: &prelude_record.lower_slices_polygons,
                lower_slices: object.lower_slices(input).unwrap_or(&[]),
                scale,
            },
            context,
        )?;
        // `this->loops->append(extrusion_coll)` only when non-empty
        // (`PerimeterGenerator.cpp:2470-2474`).
        if !outcome.collection.entities.is_empty() {
            if region.overhang_reverse.0 {
                super::super::classic::perimeter_append::reorient::reorient_perimeters(
                    &mut outcome.collection,
                    outcome.steep_overhang_contour,
                    outcome.steep_overhang_hole,
                    region.overhang_reverse_internal_only.0,
                );
            }
            perimeters.push(outcome.collection);
        }
        let boundary = walls::surface_infill_boundary(
            generated.infill_contour,
            crate::project_slice::perimeters::arachne::walls::WallCounts {
                wall_count: generated.toolpaths.len(),
                loop_number,
            },
            &spacings,
            config.scaled_resolution,
            crate::project_slice::perimeters::arachne::walls::BoundaryOverlap {
                overlaps,
                top_or_bottom: config.is_bottom_layer || config.is_topmost_layer,
            },
        )?;
        let generated = extra
            .as_ref()
            .map(|context| context.generate(&boundary.fill_surfaces))
            .transpose()?;
        fill_surfaces.extend(boundary.fill_surfaces);
        // `process_arachne:2521`, after surface append and before no-overlap.
        if let Some(generated) = generated
            && !generated.paths.is_empty()
        {
            let collection = perimeters.last_mut().unwrap();
            let mut entities = extra.as_ref().unwrap().materialize(generated.paths);
            entities.append(&mut collection.entities);
            collection.entities = entities;
            let mut remaining = Vec::new();
            for surface in fill_surfaces {
                remaining.extend(
                    difference_ex_polygons(std::slice::from_ref(&surface), &generated.filled)
                        .map_err(geometry_error)?,
                );
            }
            fill_surfaces = remaining;
        }
        fill_no_overlap.extend(boundary.fill_no_overlap);
    }
    let fill_expolygons = fill_surfaces.clone();
    let fill_surfaces = fill_surfaces
        .into_iter()
        .map(crate::project_slice::region_slices::RegionSurface::internal)
        .collect();
    Ok(PreparedLayerRegionPerimeterRecord {
        perimeters,
        thin_fills: Vec::new(),
        fill_surfaces,
        fill_expolygons,
        fill_no_overlap_expolygons: fill_no_overlap,
    })
}
