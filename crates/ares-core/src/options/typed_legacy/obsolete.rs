// Sorted alphabetically for O(log N) binary search in `is_obsolete_input`.
pub(crate) const OBSOLETE_INPUTS: [&str; 43] = [
    "acceleration",
    "adaptive_layer_height",
    "bed_size",
    "bed_temperature",
    "bed_temperature_difference",
    "bed_temperature_initial_layer",
    "can_add_auxiliary_fan",
    "can_switch_nozzle_type",
    "duplicate",
    "duplicate_grid",
    "extra_flush_volume",
    "filament_load_time",
    "filament_unload_time",
    "g0",
    "internal_bridge_support_thickness",
    "long_retraction_when_cut",
    "max_print_speed",
    "max_volumetric_speed",
    "overhang_speed_classic",
    "overhang_totally_speed",
    "print_center",
    "reduce_wall_solid_infill",
    "remove_bed_leveling",
    "remove_extrusion_calibration",
    "remove_freq_sweep",
    "retraction_distance_when_cut",
    "rotate",
    "scale",
    "silent_mode",
    "smooth_coefficient",
    "spaghetti_detector",
    "support_closing_radius",
    "support_remove_small_overhangs",
    "support_sharp_tails",
    "support_transition_line_width",
    "support_transition_speed",
    "support_with_sheath",
    "top_area_threshold",
    "tree_support_collision_resolution",
    "tree_support_with_infill",
    "wipe_tower_per_color_wipe",
    "z_hop_type",
    "z_lift_type",
];

const _: () = {
    let mut i = 1;
    while i < OBSOLETE_INPUTS.len() {
        let (a, b) = (
            OBSOLETE_INPUTS[i - 1].as_bytes(),
            OBSOLETE_INPUTS[i].as_bytes(),
        );
        let mut j = 0;
        let mut lt = a.len() < b.len();
        while j < a.len() && j < b.len() {
            if a[j] != b[j] {
                lt = a[j] < b[j];
                break;
            }
            j += 1;
        }
        assert!(lt);
        i += 1;
    }
};

pub(crate) fn is_obsolete_input(source: &str) -> bool {
    OBSOLETE_INPUTS.binary_search(&source).is_ok()
}
