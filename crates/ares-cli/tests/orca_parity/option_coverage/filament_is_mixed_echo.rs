//! PrintConfig.cpp:3545–3549,11475–11488; OrcaSlicer.cpp:4225–4244.
use crate::runner::native_stream_case;

fn check(label: &str, input: &[u8], expected_sha: &str) {
    native_stream_case::check(
        (
            "filament-is-mixed-echo",
            &[
                "PrintConfig.cpp:3545-3549,11475-11488",
                "OrcaSlicer.cpp:4225-4244 CLI::run loaded filament_count resize",
                "GCode.cpp:8244,8338-8351;Config.hpp:1994-2003,2051-2059",
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
                    .find(|line| line.starts_with("; filament_is_mixed = ")),
                Some("; filament_is_mixed = 0,0")
            );
            let activity = crate::paint_color_activity::activity(native.native.as_bytes());
            let model = activity["positive_model_XYE_by_tool"].as_object().unwrap();
            assert_eq!(model.len(), 1);
            assert!(model["1"].as_u64().unwrap() > 0);
        }),
    );
}

#[test]
fn mixed_flag_nonzero_initial_tool_strict_whole_output() {
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
fn mixed_flag_unused_hotter_filament_strict_whole_output() {
    check(
        "unused-hotter-filament",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/m140-initial-bed/unused-hotter-filament.3mf"
        )),
        "893689e2ea06b9f6de4704fc21ca6743618ac483e4e02d00eb1e50419431f6b0",
    );
}
