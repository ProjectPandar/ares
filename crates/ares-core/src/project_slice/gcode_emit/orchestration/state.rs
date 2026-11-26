//! Initial `motion::EmitState` construction for `GCode::do_export`
//! (`GCode.cpp:2533-2534,3585`).

use super::super::*;
use crate::geometry::Point;
use crate::project_slice::island_print_order::PreparedPostIslandPrintOrder;
use crate::{GenerationMetadata, SliceError};

pub(super) struct EmitStateInputs<'a> {
    pub(super) traversal: &'a PreparedPostClassicTraversal,
    pub(super) prepared: &'a PreparedPostIslandPrintOrder,
    pub(super) schedule: &'a mut layers::Schedule,
    pub(super) metadata: GenerationMetadata,
    pub(super) seed: u32,
    pub(super) brim: &'a [Option<brim::BrimPlan>],
    pub(super) skirt: Option<&'a skirt::SkirtPlan>,
    pub(super) tower_corners: &'a [Point],
    pub(super) islands: &'a footprint::FirstLayerIslands,
    pub(super) first_layer_bounds: Option<footprint::FirstLayerBounds>,
    pub(super) initial_filaments: machine::first_filaments::InitialFilaments,
    pub(super) sliced_layer_count: usize,
    pub(super) tail_layer_count: usize,
}

pub(super) fn build(inputs: EmitStateInputs<'_>) -> Result<motion::EmitState, SliceError> {
    let EmitStateInputs {
        traversal,
        prepared,
        schedule,
        metadata,
        seed,
        brim,
        skirt,
        tower_corners,
        islands,
        first_layer_bounds,
        initial_filaments,
        sliced_layer_count,
        tail_layer_count,
    } = inputs;
    // GCode.cpp:3585: one stream per emission, seeded in the GCode constructor, so it
    // is available to file start (:4230-4231) before machine start (:4381) and is not
    // the fuzzy stream.
    let template_rng = std::rc::Rc::new(std::cell::RefCell::new(
        crate::slicer_context::Mt19937::new(seed),
    ));
    let extruders = motion::ExtruderRegistry::new(
        schedule.tool_ordering.all_extruders().to_vec(),
        &traversal.resolved.views.runtime_gcode,
        traversal
            .resolved
            .views
            .full
            .project
            .print
            .nozzle_diameter
            .0
            .len(),
    );
    let writer_temperatures = machine::WriterTemperatures::new(traversal, extruders.ids());
    let options = motion::MotionOptions::from_traversal(traversal, 0);
    let small_area_flow = small_area::from_traversal(traversal)?;
    // `GCode.hpp:185-187`: origin starts at zero; real set_origin calls
    // accumulate the numeric generator cursor's local-frame translations.
    let origin = (0.0, 0.0);
    let extruder_offset = offset::extruder(traversal, initial_filaments.initial());
    let offset = (origin.0 - extruder_offset.0, origin.1 - extruder_offset.1);
    let extrusion_role_markers = options.extrusion_role_markers;
    let mut state = motion::EmitState {
        belt: motion::BeltWriter::new(&traversal.resolved.views.full.printer.remaining),
        belt_fan_layers: motion::belt_fan_layers(traversal, initial_filaments.initial()),
        total_layer_count: tail_layer_count,
        // GCode.cpp:2533-2534 initializes the region state from defaults.
        current_seam_gap: traversal.resolved.views.full.process.region.seam_gap,
        current_support_speed: traversal
            .resolved
            .views
            .full
            .process
            .object
            .support_speed
            .clone(),
        current_brim_flow_ratio: traversal
            .resolved
            .views
            .full
            .process
            .object
            .brim_flow_ratio
            .0,
        template_rng: template_rng.clone(),
        template_context: template::ExportContext::new(traversal),
        writer_temperatures,
        first_layer_hull: footprint::first_layer_convex_hull(
            traversal,
            skirt,
            brim,
            tower_corners,
            &islands.plate,
        ),
        hold_chamber_temp_for_flat_print: chamber::hold_temperature(
            traversal,
            &prepared.support_layers,
            schedule,
            brim,
        )?,
        wipe_tower: schedule.tower.take().map(|data| {
            let overlap =
                super::tower::priming_overlaps(&data, traversal, prepared, schedule, skirt);
            let center = footprint::first_layer_object_center(
                traversal,
                &schedule.instances,
                brim,
                &islands.support_spacing,
            );
            wipe_tower::Integration::new(
                data,
                traversal,
                schedule.tool_ordering.all_extruders(),
                center,
                overlap,
            )
        }),
        auto_lift: motion::auto_lift::AutoLift::prepare(traversal)
            .map_err(|error| SliceError::InvalidInput(format!("auto lift geometry: {error:?}")))?,
        extruders,
        initial_filaments,
        offset,
        origin,
        xy_offset: traversal.project.plate_origin(),
        extruder_offset,
        scale_factor: traversal.scale.factor(),
        travel_feedrate: options.first_layer_travel_feedrate,
        extrusion_feedrate: 3600.0,
        options,
        small_area_flow,
        tags: tags::Tags::of(traversal),
        spiral_vase: traversal.resolved.views.full.process.print.spiral_mode.0,
        extrusion_role_markers,
        role_change_config: {
            let runtime = &traversal.resolved.views.runtime_gcode;
            let needed = !runtime.change_extrusion_role_gcode.0.is_empty()
                || !runtime.process_change_extrusion_role_gcode.0.is_empty()
                || runtime
                    .filament_change_extrusion_role_gcode
                    .0
                    .iter()
                    .any(|template| !template.is_empty());
            needed
                .then(|| {
                    placeholders::base_config(
                        traversal,
                        metadata,
                        first_layer_bounds,
                        Some(&template_rng),
                        (sliced_layer_count, writer_temperatures),
                    )
                })
                .transpose()?
        },
        ..Default::default()
    };
    state.apply_printable_area(&traversal.resolved.views.full)?;
    Ok(state)
}
