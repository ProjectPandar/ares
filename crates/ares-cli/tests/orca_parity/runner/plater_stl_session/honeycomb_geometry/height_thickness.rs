use serde_json::json;

#[test]
fn small_module_guard_plater_geometry_matches_native() {
    super::run(
        "honeycomb-small-module-guard",
        json!({
            "layer_height": "0.08",
            "initial_layer_print_height": "0.24",
            "sparse_infill_line_width": "0.10",
            "sparse_infill_density": "40%",
            "fill_multiline": "1",
            "bottom_shell_layers": "0",
            "bottom_shell_thickness": "0",
            "initial_layer_line_width": "0.50",
            "infill_complete_top": "0",
            "sparse_infill_smooth_factor": "0%",
        }),
        false,
    );
}

fn combined(label: &str, height: &str, maximum: &str, top: &str, multiline: &str) {
    super::run(
        label,
        json!({
            "layer_height": height,
            "initial_layer_print_height": height,
            "infill_combination": "1",
            "infill_combination_max_layer_height": maximum,
            "sparse_infill_density": "20%",
            "fill_multiline": multiline,
            "infill_complete_top": top,
            "sparse_infill_smooth_factor": "0%",
        }),
        false,
    );
}

#[test]
fn two_layer_combination_plater_geometry_matches_native() {
    combined("honeycomb-two-layer", "0.20", "0.40", "0", "1");
}

#[test]
fn two_layer_cover_plater_geometry_matches_native() {
    combined("honeycomb-two-layer-cover", "0.20", "0.40", "1", "1");
}

#[test]
fn combined_multiline_cover_plater_geometry_matches_native() {
    combined(
        "honeycomb-combined-multiline-cover",
        "0.20",
        "0.40",
        "1",
        "3",
    );
}
