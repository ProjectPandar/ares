use super::*;

const NORMAL_VARIANTS: &[Replacement] = &[
    Replacement {
        from: "Normal",
        to: "Standard",
    },
    Replacement {
        from: "Big Traffic",
        to: "High Flow",
    },
];
const ZIG_ZAG: &[Replacement] = &[Replacement {
    from: "zig-zag",
    to: "rectilinear",
}];

macro_rules! rewrite {
    ($source:literal, $target:literal, $comparison:ident, $pairs:expr) => {
        scalar(
            $source,
            LegacyAction::Rewrite {
                target: $target,
                comparison: Comparison::$comparison,
                replacements: $pairs,
            },
            $target,
        )
    };
}

macro_rules! replace_all {
    ($source:literal, $kind:ident, $pairs:expr) => {
        vector(
            $source,
            LegacyAction::ReplaceAll {
                target: $source,
                replacements: $pairs,
            },
            $source,
            VectorType::$kind,
        )
    };
}

#[rustfmt::skip]
pub(crate) const EXPLICIT_RULES: &[LegacyRule] = &[
    rename("enable_wipe_tower", "enable_prime_tower"),
    rename("wipe_tower_width", "prime_tower_width"),
    rename("wiping_volume", "prime_volume"),
    rename("wipe_tower_brim_width", "prime_tower_brim_width"),
    rename("tool_change_gcode", "change_filament_gcode"),
    vector("bridge_fan_speed", LegacyAction::Rename { target: "overhang_fan_speed" }, "overhang_fan_speed", VectorType::Ints),
    rename("wipe_tower_extruder", "wipe_tower_filament"),
    rename("support_material_extruder", "support_filament"),
    rename("support_material_interface_extruder", "support_interface_filament"),
    rename("support_material_angle", "support_angle"),
    rename("support_material_enforce_layers", "enforce_support_layers"),
    vector("cooling", LegacyAction::Rename { target: "slow_down_for_layer_cooling" }, "slow_down_for_layer_cooling", VectorType::Bools),
    rename("timelapse_no_toolhead", "timelapse_type"),
    rename("sparse_infill_anchor", "infill_anchor"),
    rename("sparse_infill_anchor_max", "infill_anchor_max"),
    vector("chamber_temperatures", LegacyAction::Rename { target: "chamber_temperature" }, "chamber_temperature", VectorType::Ints),
    rename("thumbnail_size", "thumbnails"),
    rename("initial_layer_flow_ratio", "bottom_solid_infill_flow_ratio"),
    rename("ironing_direction", "ironing_angle"),
    rename("counterbole_hole_bridging", "counterbore_hole_bridging"),
    rename("prime_tower_extra_rib_length", "wipe_tower_extra_rib_length"),
    rename("prime_tower_rib_width", "wipe_tower_rib_width"),
    rename("prime_tower_fillet_wall", "wipe_tower_fillet_wall"),
    rename("extruder_clearance_max_radius", "extruder_clearance_radius"),
    rename("machine_switch_extruder_time", "machine_tool_change_time"),
    feature("infill_extruder", "sparse_infill_filament_id"),
    feature("sparse_infill_filament", "sparse_infill_filament_id"),
    feature("solid_infill_extruder", "internal_solid_filament_id"),
    feature("solid_infill_filament", "internal_solid_filament_id"),
    feature("top_solid_infill_filament", "top_surface_filament_id"),
    feature("bottom_solid_infill_filament", "bottom_surface_filament_id"),
    feature("perimeter_extruder", "outer_wall_filament_id"),
    feature("wall_filament", "outer_wall_filament_id"),
    feature("wall_filament_id", "outer_wall_filament_id"),
    feature("inner_wall_filament", "inner_wall_filament_id"),
    feature("outer_wall_filament", "outer_wall_filament_id"),
    scalar("initial_layer_print_height", LegacyAction::ConsumeIfContains { needle: "%" }, "initial_layer_print_height"),
    scalar("initial_layer_speed", LegacyAction::ConsumeIfContains { needle: "%" }, "initial_layer_speed"),
    scalar("internal_solid_infill_speed", LegacyAction::ConsumeIfContains { needle: "%" }, "internal_solid_infill_speed"),
    scalar("top_surface_speed", LegacyAction::ConsumeIfContains { needle: "%" }, "top_surface_speed"),
    scalar("support_interface_speed", LegacyAction::ConsumeIfContains { needle: "%" }, "support_interface_speed"),
    scalar("outer_wall_speed", LegacyAction::ConsumeIfContains { needle: "%" }, "outer_wall_speed"),
    scalar("support_object_xy_distance", LegacyAction::ConsumeIfContains { needle: "%" }, "support_object_xy_distance"),
    LegacyRule { wire: WireContract::scalar("only_one_wall_top", "1"), ..scalar("top_one_wall_type", LegacyAction::TopOneWall { target: "only_one_wall_top", consume: "none", replacement: "1" }, "only_one_wall_top") },
    LegacyRule { wire: WireContract { json_array: JsonArrayAllowance::ConsumeFirstPass, vector: None, empty_first_pass: EmptyValueAction::Consume, ..WireContract::scalar("wipe_tower_wall_type", "") }, ..scalar("prime_tower_rib_wall", LegacyAction::PrimeTowerRib { target: "wipe_tower_wall_type", trigger: "1", replacement: "rib" }, "wipe_tower_wall_type") },
    rewrite!("curr_bed_type", "curr_bed_type", Exact, &[Replacement { from: "SuperTack Plate", to: "Supertack Plate" }]),
    rewrite!("timelapse_type", "timelapse_type", Exact, &[Replacement { from: "2", to: "0" }]),
    LegacyRule { json_effect: Some(JsonDerivedEffect { triggers: &["hybrid(auto)"], target: "support_style", value: "tree_hybrid" }), ..rewrite!("support_type", "support_type", Exact, &[Replacement { from: "normal", to: "normal(manual)" }, Replacement { from: "tree", to: "tree(manual)" }, Replacement { from: "hybrid(auto)", to: "tree(auto)" }]) },
    rewrite!("support_base_pattern", "support_base_pattern", Exact, &[Replacement { from: "none", to: "hollow" }]),
    LegacyRule { wire: WireContract::vector("overhang_fan_threshold", VectorType::Enums), ..rewrite!("overhang_fan_threshold", "overhang_fan_threshold", Exact, &[Replacement { from: "5%", to: "10%" }]) },
    rewrite!("enable_power_loss_recovery", "enable_power_loss_recovery", AsciiCaseInsensitive, &[Replacement { from: "true", to: "enable" }, Replacement { from: "1", to: "enable" }, Replacement { from: "false", to: "disable" }, Replacement { from: "0", to: "disable" }]),
    rewrite!("ensure_vertical_shell_thickness", "ensure_vertical_shell_thickness", Exact, &[Replacement { from: "1", to: "ensure_all" }, Replacement { from: "0", to: "ensure_moderate" }]),
    rewrite!("rotate_solid_infill_direction", "solid_infill_rotate_template", Exact, &[Replacement { from: "1", to: "0,90" }, Replacement { from: "0", to: "0" }]),
    rewrite!("ironing_angle", "ironing_angle", Leading, &[Replacement { from: "-", to: "0" }]),
    rewrite!("draft_shield", "draft_shield", Exact, &[Replacement { from: "limited", to: "disabled" }]),
    rewrite!("filament_map_mode", "filament_map_mode", Exact, &[Replacement { from: "Auto", to: "Auto For Flush" }]),
    rewrite!("wall_direction", "wall_direction", Exact, &[Replacement { from: "auto", to: "ccw" }]),
    LegacyRule { json_effect: Some(JsonDerivedEffect { triggers: &["infill/outer wall/inner wall", "infill/inner wall/outer wall"], target: "is_infill_first", value: "true" }), ..scalar("wall_infill_order", LegacyAction::WallOrder { target: "wall_sequence", replacements: &[Replacement { from: "inner wall/outer wall/infill", to: "inner wall/outer wall" }, Replacement { from: "infill/inner wall/outer wall", to: "inner wall/outer wall" }, Replacement { from: "outer wall/inner wall/infill", to: "outer wall/inner wall" }, Replacement { from: "infill/outer wall/inner wall", to: "outer wall/inner wall" }, Replacement { from: "inner-outer-inner wall/infill", to: "inner-outer-inner wall" }] }, "wall_sequence") },
    replace_all!("nozzle_volume_type", Enums, NORMAL_VARIANTS),
    replace_all!("default_nozzle_volume_type", Enums, NORMAL_VARIANTS),
    replace_all!("printer_extruder_variant", Strings, NORMAL_VARIANTS),
    replace_all!("print_extruder_variant", Strings, NORMAL_VARIANTS),
    replace_all!("filament_extruder_variant", Strings, NORMAL_VARIANTS),
    replace_all!("extruder_variant_list", Strings, NORMAL_VARIANTS),
    replace_all!("extruder_type", Enums, &[Replacement { from: "DirectDrive", to: "Direct Drive" }]),
    rewrite!("sparse_infill_pattern", "sparse_infill_pattern", Exact, ZIG_ZAG),
    rewrite!("top_surface_pattern", "top_surface_pattern", Exact, ZIG_ZAG),
    rewrite!("bottom_surface_pattern", "bottom_surface_pattern", Exact, ZIG_ZAG),
    rewrite!("internal_solid_infill_pattern", "internal_solid_infill_pattern", Exact, ZIG_ZAG),
    rewrite!("ironing_pattern", "ironing_pattern", Exact, ZIG_ZAG),
    rewrite!("support_ironing_pattern", "support_ironing_pattern", Exact, ZIG_ZAG),
    vector("filament_type", LegacyAction::FilamentTokenRebuild { target: "filament_type", from: "ASA-Aero", to: "ASA-AERO" }, "filament_type", VectorType::Strings),
    LegacyRule { source: "inherits_cummulative", action: LegacyAction::DeferredProfileBookkeeping { target: Some("inherits_group"), recursive: false }, wire: WireContract::deferred(), json_effect: None, recursion: RecursionContract::SinglePass },
    LegacyRule { source: "compatible_printers_condition_cummulative", action: LegacyAction::DeferredProfileBookkeeping { target: Some("compatible_machine_expression_group"), recursive: false }, wire: WireContract::deferred(), json_effect: None, recursion: RecursionContract::SinglePass },
    LegacyRule { source: "compatible_prints_condition_cummulative", action: LegacyAction::DeferredProfileBookkeeping { target: Some("compatible_process_expression_group"), recursive: false }, wire: WireContract::deferred(), json_effect: None, recursion: RecursionContract::SinglePass },
    LegacyRule { source: "different_settings_to_system", action: LegacyAction::DeferredProfileBookkeeping { target: None, recursive: true }, wire: WireContract::deferred(), json_effect: None, recursion: RecursionContract::RecursiveBookkeeping },
];
