use serde_json::json;

fn run(label: &str, pattern: &str, smoothing: &str, multiline: &str) {
    super::fill_geometry::run(
        label,
        json!({
            "sparse_infill_pattern": pattern,
            "sparse_infill_density": "20%",
            "layer_height": "0.20",
            "initial_layer_print_height": "0.20",
            "infill_combination": "0",
            "resolution": "0.01",
            "sparse_infill_smooth_factor": smoothing,
            "fill_multiline": multiline,
        }),
        false,
    );
}

#[test]
fn hilbert_sharp_turns_plater_geometry_matches_native() {
    run("hilbert-sharp-turns", "hilbertcurve", "0%", "1");
}

#[test]
fn hilbert_rounded_turns_plater_geometry_matches_native() {
    run("hilbert-rounded-turns", "hilbertcurve", "35%", "1");
}

#[test]
fn hilbert_maximum_reach_plater_geometry_matches_native() {
    run("hilbert-maximum-reach", "hilbertcurve", "100%", "1");
}

#[test]
fn octagram_sharp_turns_plater_geometry_matches_native() {
    run("octagram-sharp-turns", "octagramspiral", "0%", "1");
}

#[test]
fn octagram_rounded_turns_plater_geometry_matches_native() {
    run("octagram-rounded-turns", "octagramspiral", "35%", "1");
}

#[test]
fn octagram_rounded_multiline_plater_geometry_matches_native() {
    run("octagram-rounded-multiline", "octagramspiral", "35%", "3");
}
