//! GCode.cpp:4019–4029,5525–5531: labelled start-temperature oracle.
use crate::{
    golden,
    runner::{application, native_api_cli::sha, native_stream_case},
};
use serde_json::json;
use std::fs;

// This section oracle does not assert whole-output parity.
fn start_through_temperature(text: &str) -> &str {
    let start = text.find("; EXECUTABLE_BLOCK_START\n").unwrap();
    let mut end = start;
    for line in text[start..].split_inclusive('\n') {
        end += line.len();
        if line.starts_with("M140 ") || line.starts_with("M190 ") {
            return &text[start..end];
        }
    }
    panic!("missing first M140/M190 in start block");
}

fn rendered_scalar(text: &str, command: &str) -> String {
    text.lines()
        .find_map(|line| {
            line.strip_prefix(command).and_then(|tail| {
                tail.split_ascii_whitespace()
                    .find_map(|word| word.strip_prefix('S'))
            })
        })
        .unwrap()
        .to_owned()
}

fn check(label: &str, input: &[u8], expected_sha: &str, temperature: &str) {
    assert_eq!(sha(input), expected_sha);
    let raw = application::settings(input).unwrap();
    let template = raw["machine_start_gcode"].as_str().unwrap();
    for command in ["M140", "M190"] {
        assert!(template.contains(&format!(
            "{command} S[bed_temperature_initial_layer_single]"
        )));
    }
    let (reference, _) = native_stream_case::qualify(
        (
            "initial-bed-temperature",
            &["bbs_3mf.cpp:4641-4646", "GCode.cpp:4019-4029,5525-5531"],
        ),
        label,
        input,
        expected_sha,
        false,
    );
    let activity = crate::paint_color_activity::activity(reference.native.as_bytes());
    let model = activity["positive_model_XYE_by_tool"].as_object().unwrap();
    assert_eq!(model.len(), 1);
    assert!(
        model["1"].as_u64().unwrap() > 0,
        "Native must actually print only filament1"
    );
    for command in ["M140 ", "M190 "] {
        assert_eq!(rendered_scalar(&reference.native, command), temperature);
    }
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(application::slice_project_plate_validated(
            input,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ));
    if let Ok(bytes) = &api {
        fs::write(reference.root.join("ares.api.gcode"), bytes).unwrap();
    }
    let path = reference.root.join("ares.cli.gcode");
    let cli = application::cli_command()
        .args(["slice", "-o"])
        .arg(&path)
        .arg(reference.root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(reference.root.join("ares.cli.stdout"), &cli.stdout).unwrap();
    fs::write(reference.root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    let cli_bytes = fs::read(&path);
    let whole_api = api
        .as_ref()
        .map_err(|error| format!("API: {error:?}"))
        .and_then(|bytes| {
            golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), bytes)
        });
    let whole_cli = cli_bytes
        .as_ref()
        .map_err(|error| format!("CLI: {error}"))
        .and_then(|bytes| {
            golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), bytes)
        });
    // Preserve the strict whole-output first difference; the assertion is scoped.
    fs::write(reference.root.join("strict-whole-comparison.json"), serde_json::to_vec_pretty(&json!({
        "input_sha256":expected_sha,"api":whole_api,"cli":whole_cli,"cli_exit":cli.status.code(),
        "whole_output_parity_claimed":false,"oracle":"LABELLED_START_THROUGH_FIRST_M140_M190_PLUS_RENDERED_SINGLE_PLACEHOLDER"
    })).unwrap()).unwrap();
    let api = api.unwrap();
    assert!(cli.status.success());
    let cli_bytes = cli_bytes.unwrap();
    assert_eq!(api, cli_bytes, "raw API/CLI equality");
    let api_text = std::str::from_utf8(&api).unwrap();
    let native_section = start_through_temperature(&reference.native);
    let api_section = start_through_temperature(api_text);
    let native_scalar = [
        rendered_scalar(&reference.native, "M140 "),
        rendered_scalar(&reference.native, "M190 "),
    ];
    let api_scalar = [
        rendered_scalar(api_text, "M140 "),
        rendered_scalar(api_text, "M190 "),
    ];
    fs::write(reference.root.join("labelled-section-comparison.json"), serde_json::to_vec_pretty(&json!({
        "oracle":"LABELLED_START_THROUGH_FIRST_M140_M190_PLUS_RENDERED_SINGLE_PLACEHOLDER",
        "whole_output_parity_claimed":false,"native_start":native_section,"api_start":api_section,
        "placeholder":"bed_temperature_initial_layer_single","native_rendered_scalar":native_scalar,
        "api_rendered_scalar":api_scalar,"native_activity":activity,
        "section_matches":native_section==api_section,"placeholder_matches":native_scalar==api_scalar
    })).unwrap()).unwrap();
    assert_eq!(
        native_section, api_section,
        "labelled start-section oracle, not whole output"
    );
    assert_eq!(
        native_scalar, api_scalar,
        "labelled rendered scalar placeholder"
    );
    let native_bed_commands: Vec<_> = reference
        .native
        .lines()
        .filter(|line| line.starts_with("M140 ") || line.starts_with("M190 "))
        .collect();
    let api_bed_commands: Vec<_> = api_text
        .lines()
        .filter(|line| line.starts_with("M140 ") || line.starts_with("M190 "))
        .collect();
    fs::write(
        reference.root.join("bed-command-comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "oracle": "ALL_M140_M190_COMMANDS",
            "whole_output_parity_claimed": false,
            "native": native_bed_commands,
            "api": api_bed_commands,
            "matches": native_bed_commands == api_bed_commands
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(native_bed_commands, api_bed_commands, "all bed commands");
}

#[test]
fn initial_bed_nonzero_filament_labelled_start() {
    check(
        "nonzero-initial-tool",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/m140-initial-bed/nonzero-initial-tool.3mf"
        )),
        "44338634ac93567c8cc10255a9825d229416c996b264c19c4dab75f78738496a",
        "73",
    );
}

// bbs_3mf.cpp:375,4641–4646 and PartPlate.cpp:267–314 at 40ecc66f7.
#[test]
fn initial_bed_plate_omitted_inherits_project_selection() {
    check(
        "plate-bed-omitted",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/bed-type-selection/omitted.3mf"
        )),
        "dfe0d43e406387a725efbbee196d61b65a8e382fae6d26e839feb3dd4bcf6000",
        "55",
    );
}

#[test]
fn initial_bed_plate_textured_pei_overrides_project_selection() {
    check(
        "plate-bed-textured-pei",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/bed-type-selection/textured-pei.3mf"
        )),
        "9945da770f311ec4e68d99a841229dd480284cf467fe3ba5fb4a69bd87fe9d20",
        "61",
    );
}

#[test]
fn initial_bed_unused_hotter_filament_labelled_start() {
    check(
        "unused-hotter-filament",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/m140-initial-bed/unused-hotter-filament.3mf"
        )),
        "893689e2ea06b9f6de4704fc21ca6743618ac483e4e02d00eb1e50419431f6b0",
        "63",
    );
}

// GCode.cpp:5532–5541,6958–6967: ordinary targets still use first-layer usage.
#[test]
fn second_layer_bed_unchanged_used_filament_ignores_unused_hotter() {
    check(
        "second-layer-unchanged-used",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/second-layer-bed/unchanged-used.3mf"
        )),
        "954b414bf9beef0c85f9934e73d61e24485de478960acd09bac3ded836572054",
        "63",
    );
}

#[test]
fn second_layer_bed_cooler_used_filament_ignores_unused_hotter() {
    check(
        "second-layer-cooler-used",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/second-layer-bed/cooler-used.3mf"
        )),
        "00fba885d384491b1c4a16ef719983de54748f410b8988a4184d96e890af55ae",
        "63",
    );
}

#[test]
fn initial_bed_plate_import_empty_is_cool() {
    check(
        "plate-bed-empty",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/bed-type-selection/empty.3mf"
        )),
        "775da9727a13149360c9405d1979485cf13d36a402fdc6788de3f4814a53bffe",
        "35",
    );
}

#[test]
fn initial_bed_plate_import_unrecognized_is_cool() {
    check(
        "plate-bed-unrecognized",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/bed-type-selection/unrecognized.3mf"
        )),
        "bdd48fd729164653805053c9c0daa72123a2a2b7a18f8e92366def519f786b51",
        "35",
    );
}

#[test]
fn initial_bed_plate_import_padded_name_is_cool() {
    check(
        "plate-bed-padded",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/bed-type-selection/padded.3mf"
        )),
        "5407de1bcf9cf71a2afb38811f7783dffe38bef0fbdfe4baa4563465cec85a42",
        "35",
    );
}

#[test]
fn initial_bed_plate_import_supertack_alias_is_concrete() {
    check(
        "plate-bed-supertack-alias",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/bed-type-selection/supertack-alias.3mf"
        )),
        "ce75ea9ba21f181fd908c02f9b6f494534a9ad4ff152679fba67a5cff6eaf5ed",
        "47",
    );
}
