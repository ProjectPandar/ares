use std::io::{Cursor, Read, Write};

use crate::{
    GCodeThumbnailFormat, GenerationMetadata, ProjectSettings, SliceError, load_project,
    slice_project,
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

const FIXTURE: &[u8] = include_bytes!(
    "../../../../../../tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf"
);

#[test]
fn absent_thumbnails_do_not_normalize_resolved_defaults() {
    let settings = parse(r#"{"thumbnails_format":"JPG"}"#);

    assert_eq!(
        settings.printer.remaining.thumbnails.as_str(),
        "48x48/PNG,300x300/PNG"
    );
    assert_eq!(
        settings.printer.remaining.thumbnails_format,
        GCodeThumbnailFormat::Jpg
    );
}

#[test]
fn canonical_and_legacy_thumbnail_inputs_both_trigger_the_composite() {
    for input in [
        r#"{"thumbnails":"16x24"}"#,
        r#"{"thumbnail_size":"16x24"}"#,
    ] {
        assert_eq!(
            parse(input).printer.remaining.thumbnails.as_str(),
            "16x24/PNG"
        );
    }
}

#[test]
fn present_empty_thumbnails_remain_empty() {
    let settings = parse(r#"{"thumbnails":"","thumbnails_format":"JPG"}"#);

    assert_eq!(settings.printer.remaining.thumbnails.as_str(), "");
}

#[test]
fn default_format_uses_only_present_input_and_explicit_item_formats_win() {
    assert_eq!(
        parse(r#"{"thumbnails":"16x24"}"#)
            .printer
            .remaining
            .thumbnails
            .as_str(),
        "16x24/PNG"
    );

    for input in [
        r#"{"thumbnails":"16x24","thumbnails_format":"QOI"}"#,
        r#"{"thumbnails_format":"QOI","thumbnails":"16x24"}"#,
    ] {
        assert_eq!(
            parse(input).printer.remaining.thumbnails.as_str(),
            "16x24/QOI"
        );
    }

    assert_eq!(
        parse(
            r#"{"thumbnails":"16x24/jpg,32x48","thumbnails_format":"QOI"}"#,
        )
        .printer
        .remaining
        .thumbnails
        .as_str(),
        "16x24/JPG, 32x48/QOI"
    );
}

#[test]
fn multiple_items_normalize_fixed_case_spacing_and_dimensions() {
    let settings = parse(
        r#"{"thumbnails":" 1.23456789mmx 2.34567891px/png,3x4/jPg, 5x6,0.0000123456789x0.000123456789/png,","thumbnails_format":"QOI"}"#,
    );

    assert_eq!(
        settings.printer.remaining.thumbnails.as_str(),
        "1.23457x2.34568/PNG, 3x4/JPG, 5x6/QOI, 1.23457e-05x0.000123457/PNG"
    );
}

#[test]
fn fixed_parser_keeps_order_duplicates_and_all_formats() {
    let settings = parse(
        r#"{"thumbnails":"1x2/png,3x4/jpg,5x6/qoi,7x8/btt_tft,9x10/colpic,11x12/png"}"#,
    );

    assert_eq!(
        settings.printer.remaining.thumbnails.as_str(),
        "1x2/PNG, 3x4/JPG, 5x6/QOI, 7x8/BTT_TFT, 9x10/COLPIC, 11x12/PNG"
    );
}

#[test]
fn fixed_parser_rejects_middle_empty_and_nonfinite_or_out_of_range_dimensions() {
    for thumbnails in [
        "1x2/png,,3x4/png",
        "not-a-numberx16",
        "NaNx16",
        "1e309x16",
        "0x16",
        "1000x16",
    ] {
        let input = format!(r#"{{"thumbnails":"{thumbnails}"}}"#);
        assert_error_contains(&input, &["invalid Orca option thumbnails"]);
    }
}

#[test]
fn incomplete_exponents_fail_but_complete_exponents_keep_trailing_junk() {
    for thumbnails in ["1ex2", "1e+x2", "1e-x2"] {
        let input = format!(r#"{{"thumbnails":"{thumbnails}"}}"#);
        assert_error_contains(&input, &["invalid Orca option thumbnails"]);
    }

    assert_eq!(
        parse(r#"{"thumbnails":"1e2mmx2px"}"#)
            .printer
            .remaining
            .thumbnails
            .as_str(),
        "100x2/PNG"
    );
}

#[test]
fn defaultfloat_selects_notation_after_six_significant_digit_rounding() {
    let settings = parse(
        r#"{"thumbnails":"0.00009999999x0.0000999988/png,999.9999x2/png"}"#,
    );

    assert_eq!(
        settings.printer.remaining.thumbnails.as_str(),
        "0.0001x9.99988e-05/PNG, 1000x2/PNG"
    );
}

#[test]
fn invalid_thumbnail_values_report_the_concrete_project_option() {
    for thumbnails in ["16", "0x16", "16x1000", "16x16/bmp"] {
        let input = format!(r#"{{"thumbnails":"{thumbnails}"}}"#);
        assert_error_contains(&input, &["invalid Orca option thumbnails"]);
    }

    assert_error_contains(
        r#"{"thumbnails":"16x16","thumbnails_format":"BMP"}"#,
        &["invalid Orca option thumbnails_format", "BMP"],
    );
}

#[test]
fn canonical_and_legacy_thumbnail_collisions_remain_strict() {
    for input in [
        r#"{"thumbnails":"16x16","thumbnail_size":"32x32"}"#,
        r#"{"thumbnail_size":"32x32","thumbnails":"16x16"}"#,
    ] {
        assert_error_contains(input, &["duplicate Orca option thumbnails"]);
    }

    assert_eq!(
        parse(r#"{"thumbnails":"16x16"}"#)
            .printer
            .remaining
            .thumbnails
            .as_str(),
        "16x16/PNG"
    );
}

#[test]
fn unreachable_legacy_inputs_remain_exact_unknown_names() {
    for source in [
        "perimeter_feed_rate",
        "wiping_volumes_matrix",
        "wiping_volumes_use_custom_matrix",
    ] {
        let input = format!(r#"{{"{source}":"value"}}"#);
        assert_error_contains(
            &input,
            &[&format!("unknown Orca project option {source}")],
        );
    }
}

#[test]
fn thumbnail_composite_does_not_change_flush_volumes_matrix() {
    let settings = parse(
        r#"{
            "flush_volumes_matrix":["0","280","280","0","0","280","280","0"],
            "thumbnails":"16x16"
        }"#,
    );

    assert_eq!(
        settings.project.print.flush_volumes_matrix.0,
        [0.0, 280.0, 280.0, 0.0, 0.0, 280.0, 280.0, 0.0]
    );
    assert_eq!(
        settings.printer.remaining.thumbnails.as_str(),
        "16x16/PNG"
    );
}

#[tokio::test]
async fn real_project_bytes_are_canonical_after_one_pass_and_reach_the_existing_boundary() {
    let project = load_project(FIXTURE).unwrap();
    let thumbnails = project.settings().printer.remaining.thumbnails.as_str();
    assert_eq!(thumbnails, "48x48/PNG, 300x300/PNG");

    let raw = project_with_thumbnails(" 16x24/png,32x48", "QOI");
    let raw = load_project(&raw).unwrap();
    assert_eq!(
        raw.settings().printer.remaining.thumbnails.as_str(),
        "16x24/PNG, 32x48/QOI"
    );
    let canonical = project_with_thumbnails("16x24/PNG, 32x48/QOI", "QOI");
    let canonical = load_project(&canonical).unwrap();
    assert_eq!(
        canonical.settings().printer.remaining.thumbnails.as_str(),
        raw.settings().printer.remaining.thumbnails.as_str()
    );

    assert_eq!(
        slice_project(
            FIXTURE,
            GenerationMetadata::deterministic(2026, 7, 13, 1, 2, 3)
        )
        .await
        .unwrap_err(),
        SliceError::ProjectSlicingIncomplete
    );
}

fn project_with_thumbnails(thumbnails: &str, format: &str) -> Vec<u8> {
    let mut source = ZipArchive::new(Cursor::new(FIXTURE)).unwrap();
    let mut destination = ZipWriter::new(Cursor::new(Vec::new()));

    for index in 0..source.len() {
        let mut entry = source.by_index(index).unwrap();
        let name = entry.name().to_owned();
        let options = SimpleFileOptions::default().compression_method(entry.compression());
        if entry.is_dir() {
            destination.add_directory(name, options).unwrap();
            continue;
        }

        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        if name == "Metadata/project_settings.config" {
            let mut settings: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            settings["thumbnails"] = serde_json::json!(thumbnails);
            settings["thumbnails_format"] = serde_json::json!(format);
            bytes = serde_json::to_vec(&settings).unwrap();
        }
        destination.start_file(name, options).unwrap();
        destination.write_all(&bytes).unwrap();
    }

    destination.finish().unwrap().into_inner()
}

fn parse(input: &str) -> ProjectSettings {
    serde_json::from_str(input).unwrap()
}

fn assert_error_contains(input: &str, expected: &[&str]) {
    let error = serde_json::from_str::<ProjectSettings>(input)
        .unwrap_err()
        .to_string();
    for fragment in expected {
        assert!(
            error.contains(fragment),
            "diagnostic omitted {fragment}: {error}"
        );
    }
}
