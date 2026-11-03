use serde_json::json;

fn run(label: &str, smoothing: &str, multiline: &str) {
    super::run(
        label,
        json!({
            "sparse_infill_pattern": "honeycomb",
            "sparse_infill_density": "20%",
            "layer_height": "0.20",
            "initial_layer_print_height": "0.20",
            "infill_combination": "0",
            "sparse_infill_smooth_factor": smoothing,
            "fill_multiline": multiline,
        }),
        false,
    );
}

#[test]
fn sharp_cells_plater_geometry_matches_native() {
    run("planar-honeycomb-sharp", "0%", "1");
}

#[test]
fn rounded_cells_plater_geometry_matches_native() {
    run("planar-honeycomb-rounded", "35%", "1");
}

#[test]
fn maximum_reach_plater_geometry_matches_native() {
    run("planar-honeycomb-maximum-reach", "100%", "1");
}

#[test]
fn rounded_multiline_plater_geometry_matches_native() {
    run("planar-honeycomb-rounded-multiline", "35%", "3");
}
