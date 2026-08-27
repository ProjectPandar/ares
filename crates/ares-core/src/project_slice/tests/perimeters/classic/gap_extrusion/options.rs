use crate::{SliceError, project_slice::perimeters::prepare_post_classic_gap_extrusion};

use super::super::super::super::support::KsrArchive;

const CONFIG: &str = "Metadata/project_settings.config";

#[test]
fn task22o14_filter_option_rejects_negative_and_nonfinite_typed_values() {
    let mut negative = KsrArchive::new();
    negative.replace_unique(
        CONFIG,
        "\"filter_out_gap_fill\": \"0\"",
        "\"filter_out_gap_fill\": \"-1\"",
    );
    match prepare_post_classic_gap_extrusion(&negative.bytes()) {
        Err(error) => assert_eq!(
            error,
            SliceError::InvalidInput("invalid Orca option filter_out_gap_fill".to_owned()),
        ),
        Ok(_) => panic!("negative filter_out_gap_fill unexpectedly succeeded"),
    }

    for value in ["nan", "inf", "-inf"] {
        let mut archive = KsrArchive::new();
        archive.replace_unique(
            CONFIG,
            "\"filter_out_gap_fill\": \"0\"",
            &format!("\"filter_out_gap_fill\": \"{value}\""),
        );
        match prepare_post_classic_gap_extrusion(&archive.bytes()) {
            Err(SliceError::InvalidInput(message)) => {
                assert!(message.contains("Orca numeric value must be finite"));
            }
            Err(error) => panic!("unexpected nonfinite error: {error:?}"),
            Ok(_) => panic!("nonfinite filter_out_gap_fill unexpectedly succeeded"),
        }
    }
}

#[test]
fn task22o14_filter_option_uses_fractional_fixed_threshold_without_rounding() {
    let baseline = prepare_post_classic_gap_extrusion(&KsrArchive::new().bytes()).unwrap();
    let baseline_count = retained_count(&baseline.objects);
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        CONFIG,
        "\"filter_out_gap_fill\": \"0\"",
        "\"filter_out_gap_fill\": \"0.0001005\"",
    );
    let filtered = prepare_post_classic_gap_extrusion(&archive.bytes()).unwrap();
    assert!(retained_count(&filtered.objects) <= baseline_count);
}

fn retained_count(
    objects: &[crate::project_slice::perimeters::classic::gap_extrusion::PreparedGapExtrusionObject],
) -> usize {
    objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
        .filter_map(|surface| surface.medial.as_ref())
        .map(|domain| domain.polylines.len())
        .sum()
}
