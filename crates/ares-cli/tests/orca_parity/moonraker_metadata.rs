//! GCode.cpp:4233-4242 producer syntax; Moonraker v0.9.3 and 4ba6fd228 metadata.
//! Missing recognition/fields, null equality, variable producer dates, altered
//! parser/footer bytes, nonrepeatable streams and changed BTT omission must fail.

use crate::{OrcaRunner, golden, native_environment, runner::native_api_cli::sha};
use serde_json::json;
use std::{fs, path::Path, process::Command};

const INPUT: &[u8] =
    include_bytes!("../../../../tests/parity/klipper-separate-jerk-precision/integer-control.3mf");
const CURRENT_PARSER: (&str, &str) = (
    "4ba6fd228f264564f6613197fd9abb18ec068cf4",
    "ae2488ff23e6ddcaf961b063506dad1a7141b38b92102bc174422ed244641d2e",
);
const LEGACY_PARSER: (&str, &str) = (
    "71f9e677b81afcc6b99dd5002f595025c38edc7b",
    "871632a1c71aa066ecf3bc88f71c978e249e88568f75f2e46b622c91afb0051c",
);

#[test]
fn moonraker_extracts_native_duration_and_filament_from_public_exports() {
    let Some(_) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let Some(current) = native_environment::binary("ARES_MOONRAKER_METADATA_PY") else {
        return;
    };
    let Some(legacy) = native_environment::binary("ARES_MOONRAKER_LEGACY_METADATA_PY") else {
        return;
    };
    let Some(python) = native_environment::binary("ARES_MOONRAKER_PYTHON") else {
        return;
    };
    let Some(pillow) = native_environment::value("ARES_MOONRAKER_PILLOW_PATH") else {
        return;
    };
    assert!(
        Path::new(&pillow).is_dir(),
        "configured Pillow path is not a directory"
    );
    let parsers = [
        ("legacy", legacy, LEGACY_PARSER),
        ("current", current, CURRENT_PARSER),
    ];
    for (_, parser, (revision, digest)) in &parsers {
        assert_eq!(
            sha(&fs::read(parser).unwrap()),
            *digest,
            "unmodified Moonraker parser {revision}"
        );
    }
    let (reference, _) = crate::runner::native_stream_case::qualify(
        (
            "moonraker-generated-at",
            &[
                "GCode.cpp:4233-4242",
                "Moonraker metadata.py 4ba6fd228:431-456",
                "Moonraker metadata.py 71f9e677b:306-324",
            ],
        ),
        "klipper-integer-control",
        INPUT,
        "3a8b3f3fbbcbdc9b7b5eb1ee13e5e08533ae8e54d60fd814be46ff45b1c02114",
        false,
    );
    let (api, cli) = reference.compare_prefix_and_maps(ares_core::GenerationMetadata::default());
    let mut processed = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_wasm::slice_project_plate_processed_bytes(
            INPUT.to_vec(),
            Some(1),
            None,
        ))
        .unwrap();
    let wasm = processed.take_gcode();
    fs::write(reference.root.join("ares.wasm.gcode"), &wasm).unwrap();
    let helper =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/support/moonraker_metadata.py");
    let paths = [
        "native.gcode",
        "ares.api.gcode",
        "ares.cli.gcode",
        "ares.wasm.gcode",
    ]
    .map(|name| reference.root.join(name));
    let python_sha = sha(&fs::read(&python).unwrap());
    let gcode_sha = json!({
        "native": sha(reference.native.as_bytes()), "api": sha(&api),
        "cli": sha(&cli), "wasm": sha(&wasm)
    });
    let extractions = parsers.map(|(mode, parser, (revision, digest))| {
        let output = reference.root.join(mode);
        fs::create_dir_all(&output).unwrap();
        fs::write(
            output.join("extractor-command.json"),
            serde_json::to_vec_pretty(&json!({
                "python": python, "python_sha256": python_sha,
                "helper": helper, "parser": parser, "parser_sha256": digest,
                "pillow_path": pillow, "exports": paths, "mode": mode,
                "moonraker_revision": revision, "gcode_sha256": gcode_sha
            }))
            .unwrap(),
        )
        .unwrap();
        let result = Command::new(&python)
            .args(["-I", "-B"])
            .arg(&helper)
            .args([mode, env!("CARGO_PKG_VERSION")])
            .arg(&parser)
            .arg(&pillow)
            .arg(&output)
            .args(&paths)
            .output()
            .unwrap();
        fs::write(output.join("extractor.stdout"), &result.stdout).unwrap();
        fs::write(output.join("extractor.stderr"), &result.stderr).unwrap();
        fs::write(
            output.join("extractor-status.json"),
            serde_json::to_vec_pretty(&json!({"exit_code": result.status.code()})).unwrap(),
        )
        .unwrap();
        (revision, output, result.status)
    });
    let comparisons = [("api", &api), ("cli", &cli), ("wasm", &wasm)].map(|(label, bytes)| {
        (
            label,
            golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), bytes),
        )
    });
    fs::write(
        reference.root.join("complete-stream-comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "comparator": "golden::compare_ordered_bytes_generator_only",
            "comparisons": comparisons, "api_cli_equal": api == cli,
            "api_wasm_equal": api == wasm
        }))
        .unwrap(),
    )
    .unwrap();
    for (revision, output, status) in extractions {
        assert!(
            status.success(),
            "Moonraker {revision} extraction at {output:?}"
        );
    }
    for (label, comparison) in comparisons {
        comparison.unwrap_or_else(|error| {
            panic!("complete {label} stream at {:?}: {error}", reference.root)
        });
    }
    assert_eq!(api, cli, "complete API/CLI equality");
    assert_eq!(api, wasm, "complete API/WASM adapter equality");
}
