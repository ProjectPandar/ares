//! PrintConfig.cpp:3118; ToolOrdering.cpp:3054–3084; Print.cpp:4656–4745.
use crate::runner::native_stream_case;

fn check(label: &str, input: &[u8], expected_sha: &str) {
    native_stream_case::check(
        (
            "filament-volume-map",
            &[
                "PrintConfig.cpp:3118;ToolOrdering.cpp:1752-1776,1892-1908,3054-3084",
                "Print.cpp:4656-4745 static map state write-back",
                "MultiNozzleUtils.cpp:get_volume_map;FilamentGroupUtils.cpp:346-358;GCode.cpp:8338-8351",
            ],
        ),
        label,
        input,
        expected_sha,
        (false, |native, _| {
            assert_eq!(
                native
                    .native
                    .lines()
                    .find(|line| line.starts_with("; filament_volume_map = ")),
                Some("; filament_volume_map = 0,0")
            );
            let activity = crate::paint_color_activity::activity(native.native.as_bytes());
            let model = activity["positive_model_XYE_by_tool"].as_object().unwrap();
            assert_eq!(model.len(), 1);
            assert!(model["1"].as_u64().unwrap() > 0);
        }),
    );
}

#[test]
fn volume_map_nonzero_initial_tool_strict_whole_output() {
    check(
        "nonzero-initial-tool",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/m140-initial-bed/nonzero-initial-tool.3mf"
        )),
        "44338634ac93567c8cc10255a9825d229416c996b264c19c4dab75f78738496a",
    );
}

#[test]
fn volume_map_unused_hotter_filament_strict_whole_output() {
    check(
        "unused-hotter-filament",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/m140-initial-bed/unused-hotter-filament.3mf"
        )),
        "893689e2ea06b9f6de4704fc21ca6743618ac483e4e02d00eb1e50419431f6b0",
    );
}
