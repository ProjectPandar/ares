use std::{
    fs,
    io::{Cursor, Read, Write},
};

use serde_json::Value;

use crate::{OrcaRunner, runner::native_api_cli::NativeCase};

const BASE: &[u8] =
    include_bytes!("../../../../../tests/parity/support-ironing/overhang-organic.3mf");
const SPARSE: &str = "sparse_infill_pattern";
const OTHER_PATTERNS: [&str; 5] = [
    "top_surface_pattern",
    "bottom_surface_pattern",
    "internal_solid_infill_pattern",
    "ironing_pattern",
    "support_ironing_pattern",
];

#[test]
fn project_settings_legacy_zigzag_patterns_slice_as_rectilinear() {
    check("legacy-zigzag-", "zig-zag", "zig-zag", "rectilinear");
}

#[test]
fn project_settings_canonical_sparse_zigzag_is_preserved() {
    check("canonical-zigzag-", "zigzag", "rectilinear", "zigzag");
}

fn project(sparse: &str, other: &str) -> Vec<u8> {
    let mut source = zip::ZipArchive::new(Cursor::new(BASE)).unwrap();
    let mut dest = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut patched = false;
    for index in 0..source.len() {
        let mut file = source.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(settings["enable_support"], "1");
            assert_eq!(settings["support_ironing"], "1");
            settings["ironing_type"] = "top".into();
            settings[SPARSE] = sparse.into();
            for key in OTHER_PATTERNS {
                settings[key] = other.into();
            }
            bytes = serde_json::to_vec_pretty(&settings).unwrap();
            patched = true;
        }
        dest.start_file(
            file.name(),
            zip::write::SimpleFileOptions::default().compression_method(file.compression()),
        )
        .unwrap();
        dest.write_all(&bytes).unwrap();
    }
    assert!(patched);
    dest.finish().unwrap().into_inner()
}

fn assert_patterns(gcode: &str, sparse: &str) {
    let (_, config) = gcode.split_once("; CONFIG_BLOCK_START\n").unwrap();
    let (config, _) = config.split_once("; CONFIG_BLOCK_END\n").unwrap();
    for (key, value) in
        std::iter::once((SPARSE, sparse)).chain(OTHER_PATTERNS.map(|key| (key, "rectilinear")))
    {
        let prefix = format!("; {key} = ");
        assert_eq!(
            config.lines().find_map(|line| line.strip_prefix(&prefix)),
            Some(value),
            "effective pattern for {key}"
        );
    }
}

fn check(label: &str, sparse: &str, other: &str, expected_sparse: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let input = project(sparse, other);
    let case = NativeCase::slice(&runner, &input, label);
    assert_patterns(&case.native, expected_sparse);
    for role in ["Bottom surface", "Internal solid infill", "Sparse infill"] {
        assert!(case.native.contains(&format!(";TYPE:{role}\n")));
    }
    let (below_top, top) = case.native.split_once(";TYPE:Top surface\n").unwrap();
    assert!(below_top.contains(";TYPE:Support interface\n"));
    assert!(below_top.contains(";TYPE:Ironing\n"), "support ironing");
    assert!(top.contains(";TYPE:Ironing\n"), "object top ironing");

    let (api, cli) = case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 10, 20, 0, 0, 0,
    ));
    for bytes in [&api, &cli] {
        assert_patterns(std::str::from_utf8(bytes).unwrap(), expected_sparse);
    }
    let warnings = fs::read_to_string(case.root.join("ares.cli.stderr")).unwrap();
    assert!(
        !warnings.contains("value \"zig-zag\""),
        "legacy pattern must not be reported as an invalid enum: {warnings}"
    );
}
