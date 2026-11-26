use std::io::{Cursor, Read, Write};

use serde_json::{Value, json};

const SUPPORT: &[u8] = include_bytes!(
    "../../../../tests/parity/support-interface-top-layers-base-connect/control.3mf"
);
const ENDER: &[u8] = include_bytes!("../../../../tests/parity/orca_cli_ender3.3mf");
const BLOCK_HOLE: &[u8] =
    include_bytes!("../../../../tests/parity/hole-to-polyhole/block-hole.3mf");
const SUPPORT_IRONING: &[u8] =
    include_bytes!("../../../../tests/parity/support-ironing/overhang-organic.3mf");
const UPSTREAM: (&str, &[&str]) = (
    "spiral-inset",
    &[
        "PrintConfig.cpp:336,474,2549–2575,2627–2633,2681–2688,7570–7589",
        "SupportParameters.hpp:151–161",
        "SupportCommon.cpp:2037–2067",
        "Fill.cpp:1010–1015,1430–1525,1593–1685,1831–1927",
        "FillSpiralInset.cpp:29–495",
    ],
);

#[test]
fn support_interface_keeps_spiral_inset() {
    compare(SUPPORT, "support_interface_pattern", "Support interface");
}

#[test]
fn top_surface_keeps_spiral_inset() {
    compare(ENDER, "top_surface_pattern", "Top surface");
}

#[test]
fn bottom_surface_keeps_spiral_inset() {
    compare(ENDER, "bottom_surface_pattern", "Bottom surface");
}

#[test]
fn internal_solid_keeps_spiral_inset() {
    compare(
        ENDER,
        "internal_solid_infill_pattern",
        "Internal solid infill",
    );
}

#[test]
fn internal_solid_hole_keeps_spiral_inset() {
    compare_patched(
        "internal_solid_hole",
        BLOCK_HOLE,
        &[
            ("internal_solid_infill_pattern", "spiralinset"),
            ("detect_narrow_internal_solid_infill", "0"),
            ("gcode_label_objects", "0"),
        ],
        None,
        &[
            "; internal_solid_infill_pattern = spiralinset",
            ";TYPE:Internal solid infill",
        ],
    );
}

#[test]
fn internal_solid_elephant_foot_density_keeps_spiral_inset() {
    compare_patched(
        "internal_solid_elephant_foot",
        ENDER,
        &[
            ("internal_solid_infill_pattern", "spiralinset"),
            ("elefant_foot_compensation_layers", "2"),
            ("elefant_foot_layers_density", "50%"),
        ],
        None,
        &[
            "; internal_solid_infill_pattern = spiralinset",
            ";TYPE:Internal solid infill",
        ],
    );
}

#[test]
fn sparse_object_override_keeps_spiral_inset() {
    compare_patched(
        "sparse_object_override",
        ENDER,
        &[],
        Some(("sparse_infill_pattern", "spiralinset")),
        &[";TYPE:Sparse infill", ";TYPE:Internal Bridge"],
    );
}

#[test]
fn sparse_hole_object_override_keeps_spiral_inset() {
    compare_patched(
        "sparse_hole_object_override",
        BLOCK_HOLE,
        &[("gcode_label_objects", "0")],
        Some(("sparse_infill_pattern", "spiralinset")),
        &[";TYPE:Sparse infill", ";TYPE:Internal Bridge"],
    );
}

#[test]
fn bottom_surface_flow_ratio_keeps_spiral_inset() {
    compare_patched(
        "bottom_surface_flow_ratio",
        ENDER,
        &[
            ("bottom_surface_pattern", "spiralinset"),
            ("bottom_solid_infill_flow_ratio", "0.9"),
        ],
        None,
        &[
            "; bottom_surface_pattern = spiralinset",
            "; bottom_solid_infill_flow_ratio = 0.9",
            ";TYPE:Bottom surface",
        ],
    );
}

#[test]
fn top_surface_hole_inward_keeps_spiral_inset() {
    compare_patched(
        "top_surface_hole_inward",
        BLOCK_HOLE,
        &[
            ("top_surface_pattern", "spiralinset"),
            ("top_surface_fill_order", "inward"),
            ("gcode_label_objects", "0"),
        ],
        None,
        &[
            "; top_surface_pattern = spiralinset",
            "; top_surface_fill_order = inward",
            ";TYPE:Top surface",
        ],
    );
}

#[test]
fn ironing_hole_keeps_spiral_inset() {
    compare_patched(
        "ironing_hole",
        BLOCK_HOLE,
        &[
            ("ironing_type", "top"),
            ("ironing_pattern", "spiralinset"),
            ("gcode_label_objects", "0"),
        ],
        None,
        &["; ironing_pattern = spiralinset", ";TYPE:Ironing"],
    );
}

#[test]
fn support_ironing_keeps_spiral_inset() {
    compare_patched(
        "support_ironing",
        SUPPORT_IRONING,
        &[("support_ironing_pattern", "spiralinset")],
        None,
        &["; support_ironing_pattern = spiralinset", ";TYPE:Ironing"],
    );
}

fn compare(fixture: &[u8], key: &str, role: &str) {
    let config_line = format!("; {key} = spiralinset");
    let role_line = format!(";TYPE:{role}");
    compare_patched(
        key,
        fixture,
        &[(key, "spiralinset")],
        None,
        &[&config_line, &role_line],
    );
}

fn compare_patched(
    label: &str,
    fixture: &[u8],
    settings_patches: &[(&str, &str)],
    object_metadata: Option<(&str, &str)>,
    required_lines: &[&str],
) {
    let project = patch_archive(fixture, settings_patches, object_metadata);
    crate::runner::native_stream_case::check(
        UPSTREAM,
        label,
        &project,
        &crate::runner::native_api_cli::sha(&project),
        (false, |reference, _| {
            for &line in required_lines {
                assert!(
                    reference.native.lines().any(|actual| actual == line),
                    "missing {line:?} in Native output at {:?}",
                    reference.root,
                );
            }
        }),
    );
}

fn patch_archive(
    fixture: &[u8],
    settings_patches: &[(&str, &str)],
    object_metadata: Option<(&str, &str)>,
) -> Vec<u8> {
    let mut input = zip::ZipArchive::new(Cursor::new(fixture)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut settings_changed = settings_patches.is_empty();
    let mut model_changed = object_metadata.is_none();
    for index in 0..input.len() {
        let mut member = input.by_index(index).unwrap();
        let options =
            zip::write::SimpleFileOptions::default().compression_method(member.compression());
        if member.is_dir() {
            output.add_directory(member.name(), options).unwrap();
            continue;
        }
        let mut bytes = Vec::new();
        member.read_to_end(&mut bytes).unwrap();
        if !settings_patches.is_empty() && member.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            for &(key, value) in settings_patches {
                assert_ne!(settings[key], value, "patch {key}={value} must differ");
                settings[key] = json!(value);
            }
            bytes = serde_json::to_vec(&settings).unwrap();
            settings_changed = true;
        } else if let Some((key, value)) = object_metadata
            && member.name() == "Metadata/model_settings.config"
        {
            let xml = String::from_utf8(bytes).unwrap();
            assert!(xml.contains("</object>"));
            let entry = format!("  <metadata key=\"{key}\" value=\"{value}\"/>\n  </object>");
            bytes = xml.replacen("</object>", &entry, 1).into_bytes();
            model_changed = true;
        }
        output.start_file(member.name(), options).unwrap();
        output.write_all(&bytes).unwrap();
    }
    assert!(settings_changed);
    assert!(model_changed);
    output.finish().unwrap().into_inner()
}
