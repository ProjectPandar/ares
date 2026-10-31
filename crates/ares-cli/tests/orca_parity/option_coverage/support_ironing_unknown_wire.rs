//! Wire-level unknown-value legs for the support-ironing domain: fixture
//! rewrites of the raw project-settings wires (bool and enum-vector) that
//! the shared domain legs do not cover. The leg runner and fixture stay in
//! `support_ironing_pattern_domain`.

use super::support_ironing_pattern_domain::{BASE, leg_with_project};
use std::io::{Cursor, Read, Write};

/// Rewrites `Metadata/project_settings.config` through an arbitrary patch
/// of its text. Verifies the fixture shape it patches.
pub(super) fn patched_project(patch: impl Fn(&mut String)) -> Vec<u8> {
    let mut reader = zip::ZipArchive::new(Cursor::new(BASE)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut text = String::from_utf8(bytes).unwrap();
            patch(&mut text);
            bytes = text.into_bytes();
        }
        writer
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::default().compression_method(file.compression()),
            )
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

/// Rewrites `support_ironing` (bool wire "1"/"0"). Seam for the coBool
/// substitution (`Config.cpp:666-668`, `looks_like_true` at
/// `Config.hpp:96-99`).
pub(super) fn ironing_bool_project(value: &str) -> Vec<u8> {
    patched_project(|text| {
        let needle = r#""support_ironing": "1""#;
        assert!(text.contains(needle), "fixture must enable support ironing");
        *text = text.replacen(needle, &format!(r#""support_ironing": "{value}""#), 1);
    })
}

/// Replaces the first `"nil"` entry after `filament_z_hop_types` with
/// `value` — key-anchored because the fixture carries fifteen `"nil"`
/// entries. Seam for the coEnums whole-vector substitution
/// (`Config.cpp:668-671`).
pub(super) fn zhop_project(value: &str) -> Vec<u8> {
    patched_project(|text| {
        let key = "\"filament_z_hop_types\":";
        let key_at = text
            .find(key)
            .expect("fixture carries filament_z_hop_types");
        let nil_at = text[key_at..]
            .find("\"nil\"")
            .expect("filament_z_hop_types carries a nil entry")
            + key_at;
        text.replace_range(nil_at..nil_at + 5, &format!("\"{value}\""));
    })
}

/// Domain: a non-"0"/"1" bool wire silently substitutes via
/// `looks_like_true` (`Config.cpp:666-668`) — "enabled" keeps ironing on.
#[test]
fn support_ironing_unknown_bool_enabled() {
    leg_with_project(
        "unknown-bool-enabled",
        "rectilinear",
        ironing_bool_project("enabled"),
    );
}

/// Domain: the literal "true" is NOT in the `looks_like_true` set
/// (`Config.hpp:96-99`: only "enabled"/"on") — it substitutes to false and
/// the ironing section disappears.
#[test]
fn support_ironing_unknown_bool_true() {
    leg_with_project(
        "unknown-bool-true",
        "rectilinear",
        ironing_bool_project("true"),
    );
}

/// Domain: an unknown coEnums entry substitutes the whole vector with the
/// option default (`Config.cpp:668-671`) — the echo carries
/// `filament_z_hop_types = Slope Lift`.
#[test]
fn support_ironing_unknown_vector_enum_zhop() {
    leg_with_project(
        "unknown-vector-enum-zhop",
        "rectilinear",
        zhop_project("Bogus Lift"),
    );
}
