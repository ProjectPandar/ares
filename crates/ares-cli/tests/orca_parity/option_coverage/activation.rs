//! Case-activation helpers for the option-domain parity sweep:
//! scope-aware enabler injection, value-compatibility co-injections, and
//! geometry-fixture selection for feature-gated options.

use serde_json::{Map, Value};

use super::domains;

#[path = "activation/support.rs"]
mod support;

pub(crate) fn inject_case(
    plan: &domains::OptionPlan,
    case: &domains::OptionCase,
    machine: &mut Map<String, Value>,
    process: &mut Map<String, Value>,
    filaments: &mut [Map<String, Value>],
) {
    // Value-compatibility co-injections that must edit BOTH presets
    // run before the scope match borrows `machine`/`process`.
    normalize_case_dependencies(plan, case, machine, process, filaments);
    // Options that only take effect when another option is enabled must
    // co-inject that enabler, otherwise the case exercises a feature the
    // slicer never activates (e.g. `ironing_pattern` under `ironing_type =
    // "no ironing"`). (option -> (enabler, activating value))
    for (enabler, value) in activating_overrides(&plan.key) {
        // Enabler scope: `input_shaping_emit`, `gcode_flavor` and the
        // resonance-avoidance keys are printer-scoped
        // (machine_envelope_config); every other enabler is process-scoped.
        let enabler_target = if matches!(
            *enabler,
            "input_shaping_emit"
                | "gcode_flavor"
                | "resonance_avoidance"
                | "min_resonance_avoidance_speed"
        ) {
            &mut *machine
        } else {
            &mut *process
        };
        enabler_target.insert((*enabler).to_owned(), Value::String((*value).to_owned()));
    }
    let target = match plan.raw_scope.as_str() {
        "printer" => machine,
        "process" => process,
        "filament" => &mut filaments[0],
        "residual" if residual_is_machine(&plan.key) => machine,
        "residual" => process,
        scope => panic!("unknown option scope {scope}"),
    };
    if let Some(value) = &case.value {
        target.insert(plan.key.clone(), value.clone());
    }
    // Upstream validation rejects some option VALUES in combination with
    // an incompatible baseline sibling — the value itself is legal, the
    // fixture is not. Co-inject the dependent setting so the oracle
    // actually slices the probed value (`Print.cpp:1740-1746` G92 guard,
    // `--use-firmware-retraction is not compatible with --wipe`).
}

fn normalize_case_dependencies(
    plan: &domains::OptionPlan,
    case: &domains::OptionCase,
    machine: &mut Map<String, Value>,
    process: &mut Map<String, Value>,
    filaments: &mut [Map<String, Value>],
) {
    super::preset_identity::activate(plan, case, machine, process, filaments);
    super::height_context::activate(&plan.key, machine, process);
    if plan.key == super::adaptive_pa_bridges::KEY {
        super::adaptive_pa_bridges::activate(process, filaments);
    }
    support::activate(&plan.key, case.value.as_ref(), process);
    if matches!(
        plan.key.as_str(),
        "support_chamber_temp_control"
            | "activate_chamber_temp_control"
            | "chamber_temperature"
            | "chamber_minimal_temperature"
            | "auxiliary_fan"
    ) {
        machine.insert(
            "support_chamber_temp_control".into(),
            serde_json::json!("1"),
        );
        machine.insert("auxiliary_fan".into(), serde_json::json!("0"));
        for (key, value) in [
            ("activate_chamber_temp_control", "1"),
            (
                "chamber_temperature",
                if plan.key == "chamber_minimal_temperature" {
                    "1500"
                } else {
                    "60"
                },
            ),
            ("chamber_minimal_temperature", "0"),
        ] {
            filaments[0].insert(key.into(), serde_json::json!([value]));
        }
        let start = machine.get_mut("machine_start_gcode").unwrap();
        *start = serde_json::json!(format!(
            ";CHAMBER_TEMPLATE_BEGIN\n{}\n;CHAMBER_MIN {{chamber_minimal_temperature[0]}}\n",
            start.as_str().unwrap()
        ));
    }
    if matches!(
        plan.key.as_str(),
        "support_air_filtration"
            | "activate_air_filtration"
            | "activate_air_filtration_during_print"
            | "activate_air_filtration_on_completion"
            | "during_print_exhaust_fan_speed"
            | "complete_print_exhaust_fan_speed"
    ) {
        machine.insert("support_air_filtration".into(), serde_json::json!("1"));
        for (key, value) in [
            ("activate_air_filtration", "1"),
            ("activate_air_filtration_during_print", "1"),
            ("activate_air_filtration_on_completion", "1"),
            ("during_print_exhaust_fan_speed", "40"),
            ("complete_print_exhaust_fan_speed", "80"),
        ] {
            filaments[0].insert(key.into(), serde_json::json!([value]));
        }
    }
    if matches!(
        plan.key.as_str(),
        "enable_long_retraction_when_cut"
            | "long_retractions_when_cut"
            | "retraction_distances_when_cut"
            | "filament_long_retractions_when_cut"
            | "filament_retraction_distances_when_cut"
    ) {
        // PrintConfig's EnableFilament gate must choose between distinct values.
        let mode_case = plan.key == "enable_long_retraction_when_cut";
        machine.insert(
            "long_retractions_when_cut".into(),
            serde_json::json!([if mode_case { "0" } else { "1" }]),
        );
        if !mode_case {
            machine.insert(
                "enable_long_retraction_when_cut".into(),
                serde_json::json!(if plan.key.starts_with("filament_") {
                    "2"
                } else {
                    "1"
                }),
            );
        }
        machine.insert(
            "retraction_distances_when_cut".into(),
            serde_json::json!(["12"]),
        );
        filaments[0].insert(
            "filament_long_retractions_when_cut".into(),
            serde_json::json!(["1"]),
        );
        filaments[0].insert(
            "filament_retraction_distances_when_cut".into(),
            serde_json::json!(["16"]),
        );
        let start = machine.get_mut("machine_start_gcode").unwrap();
        *start = serde_json::json!(format!(
            "{}\n;CUT_GATE_BEGIN\n;CUT_GATE {{long_retraction_when_cut}} {{retraction_distance_when_cut}}\n{{if long_retraction_when_cut}}G1 E-{{retraction_distance_when_cut}} F1800\n{{endif}};CUT_GATE_END",
            start.as_str().unwrap()
        ));
    }
    if plan.key == "pressure_advance" {
        // `GCode.cpp:3156-3157`: fixed PA is gated by the filament enabler.
        for (key, value) in [
            ("enable_pressure_advance", "1"),
            ("adaptive_pressure_advance", "0"),
        ] {
            filaments[0].insert(key.to_owned(), serde_json::json!([value]));
        }
    }
    let active = case
        .value
        .as_ref()
        .and_then(Value::as_str)
        .unwrap_or_default();
    if plan.key == "use_firmware_retraction" && active == "1" {
        // `wipe` is a filament-scoped option — `--use-firmware-retraction`
        // is rejected while wiping stays enabled on the material preset.
        for filament in filaments.iter_mut() {
            filament.insert("wipe".to_owned(), Value::String("0".to_owned()));
        }
    }
    if plan.key == "use_relative_e_distances" && active == "0" {
        for key in ["before_layer_change_gcode", "layer_change_gcode"] {
            for preset in [&mut *machine, &mut *process] {
                remove_absolute_reset(preset, key);
            }
        }
    }
}

fn remove_absolute_reset(preset: &mut serde_json::Map<String, Value>, key: &str) {
    if let Some(text) = preset.get(key).and_then(Value::as_str).map(str::to_owned)
        && text.contains("G92 E0")
    {
        // Replace the absolute-mode-incompatible reset with a
        // neutral comment, keeping any other lines.
        let normalized = text
            .lines()
            .map(|line| {
                if line.trim_start().starts_with("G92 E0") {
                    "; absolute-mode case: reset removed"
                } else {
                    line
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        preset.insert(key.to_owned(), Value::String(normalized));
    }
}

/// Enabler (key, activating value) pairs for options gated behind a disabled
/// feature in the baseline process preset.
/// Support-structure options need an overhanging model — on a solid cube
/// `enable_support = 1` produces no support layers and the whole family
/// stays inert. `tests/parity/overhang.stl` (T-shape, 10mm cantilevers)
/// produces ordinary + interface support geometry under the same preset.
/// Raft options stay on the cube: `raft_layers > 0` alone generates the
/// raft regardless of overhang geometry (`Print.hpp:430`).
pub(crate) fn support_geometry_option(key: &str) -> bool {
    !matches!(
        key,
        "support_air_filtration" | "support_chamber_temp_control"
    ) && (key == "enable_support"
        || key.starts_with("support_")
        || key.starts_with("tree_support_"))
}

fn activating_overrides(key: &str) -> &'static [(&'static str, &'static str)] {
    match key {
        "elefant_foot_compensation_layers" => &[
            ("elefant_foot_compensation", "0.15"),
            ("elefant_foot_layers_density", "100%"),
            ("raft_layers", "0"),
        ],
        // `Fill.cpp::group_fills` builds these flows only for Locked Zag.
        "skin_infill_line_width" | "skeleton_infill_line_width" => &[
            ("sparse_infill_pattern", "lockedzag"),
            ("sparse_infill_density", "40%"),
        ],
        "hot_plate_temp_initial_layer" => &[("curr_bed_type", "High Temp Plate")],
        // `Print.cpp:575-585`: infinite skirts bypass the configured height.
        "skirt_height" => &[("skirt_loops", "1"), ("draft_shield", "disabled")],
        // Ironing sub-options only run when `ironing_type != "no ironing"`.
        "ironing_flow"
        | "ironing_spacing"
        | "ironing_speed"
        | "ironing_pattern"
        | "ironing_inset"
        | "ironing_angle"
        | "ironing_angle_fixed"
        | "filament_ironing_flow"
        | "filament_ironing_speed"
        | "filament_ironing_spacing"
        | "filament_ironing_inset"
        | "ironing_fan_speed" => &[("ironing_type", "top")],
        // Fuzzy-skin geometry params only run when `fuzzy_skin != "none"`.
        "fuzzy_skin_point_dist" | "fuzzy_skin_point_type" => &[("fuzzy_skin", "external")],
        "fuzzy_skin_first_layer"
        | "fuzzy_skin_layers_between_ripple_offset"
        | "fuzzy_skin_mode"
        | "fuzzy_skin_noise_type"
        | "fuzzy_skin_point_distance"
        | "fuzzy_skin_ripple_offset"
        | "fuzzy_skin_ripples_per_layer"
        | "fuzzy_skin_thickness" => &[("fuzzy_skin", "external")],
        // Coherent-noise params are consumed only by the non-ripple noise
        // modules (`FuzzySkin.cpp:296-301` bypasses them for ripple), so
        // the domain must activate a coherent noise type.
        "fuzzy_skin_octaves" | "fuzzy_skin_persistence" | "fuzzy_skin_scale" => &[
            ("fuzzy_skin", "external"),
            ("fuzzy_skin_noise_type", "perlin"),
        ],
        // Input-shaping params only emit `M593`/`SET_INPUT_SHAPER` when
        // `input_shaping_emit` is on AND the flavor is not legacy
        // (`GCode.cpp:3991`). Upstream maps `"marlin"` → `gcfMarlinLegacy`
        // and `"marlin2"` → `gcfMarlinFirmware`
        // (`PrintConfig.cpp:164-168`) — the Ender-3 baseline is legacy, so
        // these cases must switch the flavor to marlin2 to exercise the
        // emission branch at all.
        "input_shaping_damp_x"
        | "input_shaping_damp_y"
        | "input_shaping_freq_x"
        | "input_shaping_freq_y"
        | "input_shaping_type" => &[("input_shaping_emit", "1"), ("gcode_flavor", "marlin2")],
        // `input_shaping_emit`'s own 0/1 domain: the `1` case emits
        // `M593` only on a non-legacy flavor.
        "input_shaping_emit" => &[("gcode_flavor", "marlin2")],
        // Resonance avoidance only rewrites external-perimeter speeds when
        // enabled (`GCode.cpp:6621-6622`) and the speed lies below the band
        // max. The fixture outer wall (25mm/s) sits under the default 70mm/s
        // min, so the enable domain lowers the min to put it inside the band.
        "min_resonance_avoidance_speed" => &[("resonance_avoidance", "1")],
        // The max domain only probes 0 and a seeded 3.523 (min-only schema),
        // both under the fixture wall: slow the wall to 1.5mm/s and drop the
        // min to 1 so the seeded band [1, 3.523] clamps it to the min.
        "max_resonance_avoidance_speed" => &[
            ("resonance_avoidance", "1"),
            ("min_resonance_avoidance_speed", "1"),
            ("outer_wall_speed", "1.5"),
        ],
        "resonance_avoidance" => &[("min_resonance_avoidance_speed", "20")],
        // Support structure sub-options only run when supports generate
        // (`support_layers`/`raft`/`tree_support_*` all gate on
        // `enable_support`). `support_type`/`support_style` select the
        // generator and stay active on the cube only when supports are
        // enabled AND the model overhangs — the sweep uses
        // `tests/parity/overhang.stl` for these keys (see
        // `support_geometry_option`).
        "support_angle"
        | "support_base_pattern"
        | "support_base_pattern_spacing"
        | "support_bottom_interface_spacing"
        | "support_bottom_z_distance"
        | "support_critical_regions_only"
        | "support_expansion"
        | "support_filament"
        | "support_flow_ratio"
        | "support_interface_bottom_layers"
        | "support_interface_filament"
        | "support_interface_flow_ratio"
        | "support_interface_loop_pattern"
        | "support_interface_not_for_body"
        | "support_interface_pattern"
        | "support_interface_spacing"
        | "support_interface_speed"
        | "support_interface_top_layers"
        | "support_ironing"
        | "support_ironing_flow"
        | "support_ironing_pattern"
        | "support_ironing_spacing"
        | "support_line_width"
        | "support_material_angle"
        | "support_material_enforce_layers"
        | "support_material_extruder"
        | "support_material_interface_extruder"
        | "support_material_interface_fan_speed"
        | "support_multi_bed_types"
        | "support_object_first_layer_gap"
        | "support_object_skip_flush"
        | "support_object_xy_distance"
        | "support_on_build_plate_only"
        | "support_parallel_printheads"
        | "support_remove_small_overhang"
        | "support_speed"
        | "support_style"
        | "support_threshold_angle"
        | "support_threshold_overlap"
        | "support_top_z_distance"
        | "support_type" => &[("enable_support", "1")],
        // Raft params gate on `raft_layers > 0` (`Print.hpp:430`), not
        // `enable_support`; a raft does not need support enabled.
        "raft_contact_distance"
        | "raft_expansion"
        | "raft_first_layer_density"
        | "raft_first_layer_expansion" => &[("raft_layers", "2")],
        // Gap-fill params only run when gap filling is enabled.
        "gap_fill_target" => &[("gap_infill_type", "everywhere")],
        // Wrapping templates and their exclude area only emit when
        // detection is on (`GCode.cpp:5302-5305`).
        "wrapping_detection_gcode" | "wrapping_exclude_area" => {
            &[("enable_wrapping_detection", "1")]
        }
        _ => &[],
    }
}

fn residual_is_machine(key: &str) -> bool {
    matches!(
        key,
        "deretraction_speed"
            | "extruder_offset"
            | "max_layer_height"
            | "min_layer_height"
            | "nozzle_diameter"
            | "nozzle_volume_type"
            | "retract_before_wipe"
            | "retract_length_toolchange"
            | "retract_lift_above"
            | "retract_lift_below"
            | "retract_restart_extra"
            | "retract_restart_extra_toolchange"
            | "retraction_length"
            | "retraction_minimum_travel"
            | "retraction_speed"
            | "wipe"
            | "wipe_distance"
            | "z_hop"
    )
}
