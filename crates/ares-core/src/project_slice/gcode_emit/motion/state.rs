mod config;
mod layer;
pub(in crate::project_slice::gcode_emit) use layer::{LayerBegin, begin_layer};
mod object_labels;
pub(in crate::project_slice::gcode_emit) use object_labels::*;

use super::{MotionOptions, arc, extruder_state, set_accel_and_jerk};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::project_slice::gcode_emit) enum LiftMode {
    #[default]
    Normal,
    Slope,
    Spiral,
}

use super::shared_reversals::SharedEntityReversals;

#[derive(Default)]
pub(in crate::project_slice::gcode_emit) struct EmitState {
    pub(in crate::project_slice::gcode_emit) global_brim_done: bool,
    pub(in crate::project_slice::gcode_emit) emitted_brim_owners: std::collections::BTreeSet<usize>,
    pub(in crate::project_slice::gcode_emit) contour_z_too_low: bool,
    pub(in crate::project_slice::gcode_emit) last_path_z_contoured: bool,
    pub(in crate::project_slice::gcode_emit) total_layer_count: usize,
    pub(in crate::project_slice::gcode_emit) template_rng: super::super::value::TemplateRng,
    pub(in crate::project_slice::gcode_emit) hold_chamber_temp_for_flat_print: bool,
    pub(in crate::project_slice::gcode_emit) first_layer_hull: Vec<crate::geometry::Point>,
    pub(in crate::project_slice::gcode_emit) wipe_tower:
        Option<super::super::wipe_tower::Integration>,
    pub(in crate::project_slice::gcode_emit) adaptive_pa_flow: Option<f64>,
    pub(in crate::project_slice::gcode_emit) adaptive_pa_emitted: bool,
    pub(in crate::project_slice::gcode_emit) adaptive_pa_last_flow: f64,
    pub(in crate::project_slice::gcode_emit) adaptive_pa_resets: Vec<(usize, f64)>,
    pub(in crate::project_slice::gcode_emit) adaptive_pa_layer: usize,
    pub(in crate::project_slice::gcode_emit) power_loss_recovery:
        crate::gcode_power_loss_recovery::PowerLossRecoveryState,
    pub(in crate::project_slice::gcode_emit) x: f64,
    pub(in crate::project_slice::gcode_emit) y: f64,
    pub(in crate::project_slice::gcode_emit) extruder: extruder_state::ExtruderState,
    pub(in crate::project_slice::gcode_emit) extruders: super::ExtruderRegistry,
    pub(in crate::project_slice::gcode_emit) initial_filaments:
        crate::project_slice::gcode_emit::machine::first_filaments::InitialFilaments,
    pub(in crate::project_slice::gcode_emit) offset: (f64, f64),
    /// `GCode::point_to_gcode` (`GCode.cpp:8124-8127`) evaluates
    /// `unscale(point) + m_origin - extruder_offset` strictly left-to-right;
    /// folding the extruder offset into `offset` first rounds one ULP
    /// differently across 0.0005 formatting boundaries (mHlqqz X149.029).
    pub(in crate::project_slice::gcode_emit) origin: (f64, f64),
    /// Upstream uses origin zero while emitting skirt/brim, including the
    /// travel before each loop. Stored cursors and wipe points stay local.
    pub(in crate::project_slice::gcode_emit) plate_loop_origin: bool,
    pub(in crate::project_slice::gcode_emit) extruder_offset: (f64, f64),
    /// `m_curr_extruder_id != -1` (`GCode.cpp:8138-8144`): the extruder
    /// offset only applies to `gcode_to_point` write-backs once the
    /// initial `set_extruder` ran. BBL's `init_extruder` precedes its
    /// filament-start (`GCode.cpp:3144`); the non-BBL extruder is
    /// selected inside `set_extruder` AFTER rendering filament-start
    /// (`GCode.cpp:7736-7746`).
    pub(in crate::project_slice::gcode_emit) extruder_selected: bool,
    pub(in crate::project_slice::gcode_emit) current_hotend: i32,
    pub(in crate::project_slice::gcode_emit) toolchange_count: usize,
    pub(in crate::project_slice::gcode_emit) start_gcode_filament: Option<usize>,
    pub(in crate::project_slice::gcode_emit) scale_factor: f64,
    pub(in crate::project_slice::gcode_emit) travel_feedrate: f64,
    pub(in crate::project_slice::gcode_emit) extrusion_feedrate: f64,
    pub(in crate::project_slice::gcode_emit) current_feedrate: f64,
    pub(in crate::project_slice::gcode_emit) options: MotionOptions,
    /// Current region field in GCode::m_config; object applies do not reset it.
    pub(in crate::project_slice::gcode_emit) current_seam_gap: crate::FloatOrPercent,
    /// Current object field in GCode::m_config, applied before each instance's body.
    pub(in crate::project_slice::gcode_emit) current_brim_flow_ratio: f64,
    pub(in crate::project_slice::gcode_emit) current_support_speed: f64,
    pub(in crate::project_slice::gcode_emit) auto_lift: super::auto_lift::AutoLift,
    pub(in crate::project_slice::gcode_emit) active_layer_print_z: f64,
    pub(in crate::project_slice::gcode_emit) small_area_flow:
        crate::extrusions::SmallAreaInfillFlowCompensation,
    /// Object-level shared orientation state for chained emission: the port
    /// of upstream's in-place reversals of the SHARED layer entities
    /// (`GCode.cpp:6164,6201` over the pointer vector; `ShortestPath.cpp:
    /// 1040-1052` `out.back()->reverse()`), which later islands/instances
    /// of the same source object observe. Keyed by
    /// `(source_object_index, layer)` — never cross-layer, never
    /// cross-object; deterministic and independent of parallelism.
    pub(in crate::project_slice::gcode_emit) shared_reversals: SharedEntityReversals,
    /// Cached `shares_print_toolpaths` representatives per source object
    /// (`Print.cpp:2240-2251`) — the object whose shared layers later
    /// identical objects observe.
    pub(in crate::project_slice::gcode_emit) print_sharing:
        std::collections::BTreeMap<(usize, usize), (usize, usize)>,
    pub(in crate::project_slice::gcode_emit) small_area_gate: super::super::small_area::Gate,
    /// Active layer id — upstream's `layer->id()`
    /// (`GCode.cpp:5689`, `GCode.hpp:671-676`): raft layers occupy ids
    /// `0..raft_count`, so the first object layer after a raft has id
    /// `raft_count`, not 0. First-layer speed/accel/jerk gates read this
    /// via `on_first_layer` (`id()==0 && bottom_z≈0`) and the over-raft
    /// flag below.
    pub(in crate::project_slice::gcode_emit) layer_index: usize,
    /// Current `m_layer`'s `id()==0 && abs(bottom_z)<EPSILON`, distinct
    /// from the global layer counter (`GCode.hpp:670`).
    pub(in crate::project_slice::gcode_emit) on_first_layer: bool,
    /// Global merged chunk counter (`GCode::m_layer_index`).
    pub(in crate::project_slice::gcode_emit) layer_chunk_index: usize,
    /// `m_object_layer_over_raft` (`GCode.cpp:5339-5344`): the first
    /// object layer printed over a raft interface — applies
    /// `initial_layer_speed`/`initial_layer_infill_speed` to non-bottom
    /// roles (`GCode.cpp:6564-6570`).
    pub(in crate::project_slice::gcode_emit) object_layer_over_raft: bool,
    pub(in crate::project_slice::gcode_emit) spiral_vase: bool,
    pub(in crate::project_slice::gcode_emit) spiral_vase_layer: bool,
    pub(in crate::project_slice::gcode_emit) positioned: bool,
    /// The Z the machine start/filament g-code left the nozzle at; mirrors
    /// `GCodeWriter::m_pos(2)` for the `change_layer` `will_move_z` gate.
    pub(in crate::project_slice::gcode_emit) writer_z: Option<f64>,
    /// `GCode.hpp:607-608`: numeric cursor defaults to zero independently of
    /// its defined flag; writer-only inward moves change neither field.
    pub(in crate::project_slice::gcode_emit) last_scaled_position: (i64, i64),
    pub(in crate::project_slice::gcode_emit) last_position_defined: bool,
    pub(in crate::project_slice::gcode_emit) last_feature: Option<&'static str>,
    /// `m_last_notgapfill_extrusion_role` (`GCode.cpp:7255-7257`): the last
    /// emitted feature that is not gap fill — the `retract_lift_enforce`
    /// top-surface check (`GCode.cpp:7682`) reads this, not `last_feature`.
    pub(in crate::project_slice::gcode_emit) last_notgapfill_feature: Option<&'static str>,
    /// `;_EXTRUSION_ROLE:n` emission gate (`m_enable_extrusion_role_markers`,
    /// `GCode.cpp:2544` — on when the PressureEqualizer is in the pipeline).
    pub(in crate::project_slice::gcode_emit) extrusion_role_markers: bool,
    pub(in crate::project_slice::gcode_emit) last_extrusion_role: Option<u32>,
    /// Frozen `base_config` for the role-change templates
    /// (`GCode.cpp:6752-6758` merges DynamicConfig over the global
    /// placeholder parser config).
    pub(in crate::project_slice::gcode_emit) role_change_config:
        Option<crate::project_slice::gcode_emit::value::Config>,
    pub(in crate::project_slice::gcode_emit) last_width: Option<f32>,
    pub(in crate::project_slice::gcode_emit) last_height: Option<f32>,
    pub(in crate::project_slice::gcode_emit) last_acceleration: Option<u32>,
    pub(in crate::project_slice::gcode_emit) last_jerk: Option<f64>,
    pub(in crate::project_slice::gcode_emit) layer_z: f64,
    /// `m_max_layer_z` (`GCode.cpp:4630`): running float-domain max of the
    /// RAW print z (no `z_offset`), feeding the `{max_layer_z}` wrapping
    /// template — upstream supplies the float-rounded value
    /// (`GCode.cpp:5221`).
    pub(in crate::project_slice::gcode_emit) max_layer_z: f32,
    pub(in crate::project_slice::gcode_emit) source_layer_z: f64,
    pub(in crate::project_slice::gcode_emit) scarf_z: Option<f64>,
    pub(in crate::project_slice::gcode_emit) wipe_path: Vec<arc::Point>,
    pub(in crate::project_slice::gcode_emit) wipe_start: Option<arc::Point>,
    pub(in crate::project_slice::gcode_emit) lifted: bool,
    /// Upstream `m_lifted` as a DISTANCE (`GCodeWriter.cpp:770-830`):
    /// reduced by in-band z deltas, cleared on real z moves — the bool
    /// above cannot represent partial reduction.
    pub(in crate::project_slice::gcode_emit) lifted_amount: f64,
    pub(in crate::project_slice::gcode_emit) template_lifted: bool,
    pub(in crate::project_slice::gcode_emit) pending_lift: Option<LiftMode>,
    pub(in crate::project_slice::gcode_emit) part_fan_speed: u8,
    pub(in crate::project_slice::gcode_emit) physical_fan_speed: u8,
    pub(in crate::project_slice::gcode_emit) overhang_fan_active: bool,
    pub(in crate::project_slice::gcode_emit) overhang_fan_marker_layer: Option<usize>,
    pub(in crate::project_slice::gcode_emit) internal_bridge_fan_active: bool,
    pub(in crate::project_slice::gcode_emit) internal_bridge_fan_marker_layer: Option<usize>,
    pub(in crate::project_slice::gcode_emit) support_interface_fan_marker_layer: Option<usize>,
    pub(in crate::project_slice::gcode_emit) ironing_fan_marker_layer: Option<usize>,
    pub(in crate::project_slice::gcode_emit) pending_object_start: Option<(u32, [u8; 12])>,
    pub(in crate::project_slice::gcode_emit) filament_instances_code: Option<[u8; 12]>,
    pub(in crate::project_slice::gcode_emit) layer_object_labels:
        Option<super::super::object::LayerLabels>,
    pub(in crate::project_slice::gcode_emit) last_travel_acceleration: Option<u32>,
    pub(in crate::project_slice::gcode_emit) pending_exclude_start: Option<String>,
    pub(in crate::project_slice::gcode_emit) pending_exclude_end: Option<String>,
    pub(in crate::project_slice::gcode_emit) pending_object_stop_label: Option<u32>,
    pub(in crate::project_slice::gcode_emit) tags: super::super::tags::Tags,
    /// The one-shot routing disable armed after the first-layer skirt
    // (`disable_once`, `GCode.cpp:4448-4450`: a straight travel to the
    // first object point).
    pub(in crate::project_slice::gcode_emit) avoid_crossing_disabled_once: bool,
    /// `use_external_mp_once` (`AvoidCrossingPerimeters.hpp:21`): armed
    /// when a new object copy starts (`GCode.cpp:5380-5384`), the first
    /// travel routes along the chunk-wide hole boundary; cleared with the
    /// other once-modifiers after the emitted travel (`GCode.cpp:7431`).
    pub(in crate::project_slice::gcode_emit) use_external_mp_once: bool,
    pub(in crate::project_slice::gcode_emit) pending_layer_retract: bool,
    pub(in crate::project_slice::gcode_emit) layer_change_travel_pending: bool,
    pub(in crate::project_slice::gcode_emit) traditional_timelapse: bool,
    /// `has_insert_timelapse_gcode` is a `process_layer` local upstream
    /// (`GCode.cpp:4653`) — reset at every layer, set by the traditional
    /// timelapse insert (`GCode.cpp:5289-5297`, ported at the toolchange
    /// boundary) and the per-layer interlude.
    pub(in crate::project_slice::gcode_emit) has_insert_timelapse_gcode: bool,
    /// `ToolOrdering::most_used_extruder` (`ToolOrdering.cpp:955-981`),
    /// resolved once from the layer tool ordering.
    pub(in crate::project_slice::gcode_emit) most_used_extruder: usize,
    /// Per-layer avoid-crossing routing boundary, built lazily on the first
    /// routed travel and dropped by `begin_layer`
    /// (`AvoidCrossingPerimeters::init_layer`).
    pub(in crate::project_slice::gcode_emit) avoid_boundary:
        Option<std::rc::Rc<super::path::Boundary>>,
    /// Curled-perimeter tracker feeding the overhang estimator's
    /// artificial distance (`SupportSpotsGenerator.cpp:141-196`).
    pub(in crate::project_slice::gcode_emit) curl: super::overhang::CurlTracker,
    /// `ppi.failed_templates` (`GCode.cpp:3920-3933`): first failure per
    /// template name; export aborts after emit (`GCode.cpp:3637-3645`).
    pub(in crate::project_slice::gcode_emit) failed_templates: Vec<(String, String)>,
}

impl EmitState {
    pub(in crate::project_slice::gcode_emit) fn record_template_failure(
        &mut self,
        name: &str,
        error: &str,
    ) {
        if !self.failed_templates.iter().any(|(key, _)| key == name) {
            self.failed_templates
                .push((name.to_owned(), error.to_owned()));
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::project_slice::gcode_emit) struct LayerGeometry<'a> {
    /// Per-layer nearest-seam penalty data for the Nearest mode — the
    /// layer's candidates with emit-time gaussian distance selection
    /// (`SeamPlacer.cpp:1500-1560`).
    pub(in crate::project_slice::gcode_emit) nearest_seam_penalties:
        Option<&'a crate::project_slice::island_print_order::NearestSeamLayer>,
    /// `staggered_inner_seams` from the owning object (`SeamPlacer.cpp:1601`).
    pub(in crate::project_slice::gcode_emit) staggered_inner: bool,
    pub(in crate::project_slice::gcode_emit) internal_surfaces:
        &'a [&'a crate::project_slice::region_slices::RegionSurface],
    /// Active SupportLayer containment, independent of pair-selected planner geometry.
    pub(in crate::project_slice::gcode_emit) support_islands:
        Option<super::support::LayerAreas<'a>>,
    pub(in crate::project_slice::gcode_emit) scale: crate::geometry::CoordinateScale,
    pub(in crate::project_slice::gcode_emit) previous_layer_boundary:
        Option<&'a crate::geometry::LineDistanceTree<'a>>,
    /// Current layer's merged slices plus the perimeter spacing and top
    /// surfaces — the avoid-crossing boundary inputs
    /// (`AvoidCrossingPerimeters.cpp:1099-1134`).
    pub(in crate::project_slice::gcode_emit) avoid_crossing: AvoidCrossingGeometry<'a>,
}

#[derive(Clone, Copy)]
pub(in crate::project_slice::gcode_emit) struct AvoidCrossingGeometry<'a> {
    pub(in crate::project_slice::gcode_emit) layer_slices: &'a [crate::geometry::ExPolygon],
    pub(in crate::project_slice::gcode_emit) perimeter_spacing: f32,
    pub(in crate::project_slice::gcode_emit) external_perimeter_width: f32,
    pub(in crate::project_slice::gcode_emit) top_surfaces: &'a [&'a crate::geometry::ExPolygon],
    /// Every print object's slices at this print z and the cross-object
    /// average perimeter spacing, feeding the EXTERNAL motion planner's
    /// hole boundary (`get_boundary_external`,
    /// `AvoidCrossingPerimeters.cpp:1137-1189`). Already translated once
    /// per print instance (`:1172-1177`), i.e. plate coordinates —
    /// `layers/group.rs` `boundary_slices`.
    pub(in crate::project_slice::gcode_emit) chunk_slices: &'a [crate::geometry::ExPolygon],
    pub(in crate::project_slice::gcode_emit) chunk_perimeter_spacing: f64,
}

impl EmitState {
    pub(in crate::project_slice::gcode_emit) fn extrusion_totals(
        &self,
    ) -> crate::project_slice::gcode_emit::layer_gcode::ExtrusionTotals {
        let used_filament = if self.options.single_extruder_multi_material {
            self.extruder.filament_used
        } else {
            self.extruder.filament_used + self.extruder.retracted_amount
        };
        crate::project_slice::gcode_emit::layer_gcode::ExtrusionTotals {
            diameter: 1.75,
            density: 1.24,
            used_filament,
        }
    }

    /// Track an emitted Z coordinate: `writer_z` mirrors
    /// `GCodeWriter::m_pos(2)` (`GCodeWriter.cpp`) — every emitted move
    /// carrying a Z word (lifts, descends, scarf/spiral Z-in-extrusion,
    /// combined `G1 X..Y..Z..` travels) advances it. The eager-lift gate and
    /// target (`GCodeWriter.cpp:659-676`) and the layer-change
    /// `will_move_z` check (`GCode.cpp:4417`) both read this current Z.
    pub(in crate::project_slice::gcode_emit) fn record_z(&mut self, z: f64) {
        self.writer_z = Some(z);
    }
}

pub(in crate::project_slice::gcode_emit) fn begin_path_travel(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    destination_feature: &str,
    travel_distance: f64,
) {
    let acceleration = if state.options.default_acceleration == 0 {
        None
    } else if state.layer_index == 0 {
        (state.options.initial_layer_travel_acceleration > 0)
            .then_some(state.options.initial_layer_travel_acceleration)
    } else if travel_distance < state.options.retraction_minimum_travel {
        match destination_feature {
            "Overhang wall" => {
                (state.options.bridge_acceleration > 0).then_some(state.options.bridge_acceleration)
            }
            "Outer wall" => (state.options.outer_wall_acceleration > 0)
                .then_some(state.options.outer_wall_acceleration),
            _ => None,
        }
        .or((state.options.travel_acceleration > 0).then_some(state.options.travel_acceleration))
    } else {
        (state.options.travel_acceleration > 0).then_some(state.options.travel_acceleration)
    };
    let jerk = if state.options.default_jerk <= 0.0 {
        0.0
    } else if state.layer_index == 0 {
        state.options.travel_jerk
    } else if travel_distance < state.options.retraction_minimum_travel
        && matches!(destination_feature, "Outer wall" | "Overhang wall")
        && state.options.outer_wall_jerk > 0.0
    {
        state.options.outer_wall_jerk
    } else {
        state.options.travel_jerk
    };
    set_accel_and_jerk(output, state, acceleration.unwrap_or(0), jerk, true);
}
