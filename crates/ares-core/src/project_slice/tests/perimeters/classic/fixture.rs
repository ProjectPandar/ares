use crate::{SliceError, project_slice::perimeters::prepare_post_classic_prelude, slice_project};

use super::super::super::support::{KsrArchive, ksr_project, metadata};

const NORMAL_PRINTABLE_AREA: &str = concat!(
    "\t\"printable_area\": [\r\n",
    "\t\t\"0x0\",\r\n",
    "\t\t\"256x0\",\r\n",
    "\t\t\"256x256\",\r\n",
    "\t\t\"0x256\"\r\n",
    "\t]",
);
const LARGE_PRINTABLE_AREA: &str = concat!(
    "\t\"printable_area\": [\r\n",
    "\t\t\"0x0\",\r\n",
    "\t\t\"2148x0\",\r\n",
    "\t\t\"2148x256\",\r\n",
    "\t\t\"0x256\"\r\n",
    "\t]",
);

fn first_record(project: impl AsRef<[u8]>) -> (i64, bool) {
    prepare_post_classic_prelude(project)
        .unwrap()
        .objects
        .into_iter()
        .flat_map(|object| object.into_parts().1)
        .flatten()
        .map(|record| (record.external_to_internal_spacing, record.has_gap_fill))
        .next()
        .unwrap()
}

fn first_record_resolution(project: impl AsRef<[u8]>) -> f64 {
    prepare_post_classic_prelude(project)
        .unwrap()
        .objects
        .into_iter()
        .flat_map(|object| object.into_parts().1)
        .flatten()
        .map(|record| record.surface_simplify_resolution)
        .next()
        .unwrap()
}

#[test]
fn task22o1_fixture_prelude_runs_at_both_supported_coordinate_scales() {
    let normal = prepare_post_classic_prelude(ksr_project()).unwrap();
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        NORMAL_PRINTABLE_AREA,
        LARGE_PRINTABLE_AREA,
    );
    let large = prepare_post_classic_prelude(archive.bytes()).unwrap();
    assert_ne!(normal.scale, large.scale);
    assert_eq!(normal.objects.len(), large.objects.len());
    let normal_widths = normal
        .objects
        .into_iter()
        .flat_map(|object| object.into_parts().1)
        .flatten()
        .take(2)
        .map(|record| record.external_width)
        .collect::<Vec<_>>();
    let large_widths = large
        .objects
        .into_iter()
        .flat_map(|object| object.into_parts().1)
        .flatten()
        .take(2)
        .map(|record| record.external_width)
        .collect::<Vec<_>>();
    assert_eq!(normal_widths, [500_000, 419_999]);
    assert_eq!(large_widths, [49_999, 41_999]);
}

#[tokio::test]
async fn task22o1_fixture_public_lifecycle_consumes_prelude_then_stays_incomplete() {
    assert_eq!(
        slice_project(ksr_project(), metadata()).await.unwrap_err(),
        SliceError::ProjectSlicingIncomplete
    );
}

#[test]
fn task22o1_fixture_precise_spacing_and_gap_enablement_come_from_3mf_options() {
    let baseline = first_record(ksr_project());
    assert_eq!(baseline.0, 500_000);

    let mut imprecise = KsrArchive::new();
    imprecise.replace_unique(
        "Metadata/project_settings.config",
        "\"precise_outer_wall\": \"1\"",
        "\"precise_outer_wall\": \"0\"",
    );
    let imprecise = first_record(imprecise.bytes());
    assert_ne!(baseline.0, imprecise.0);
    assert_eq!(baseline.1, imprecise.1);

    let mut no_gaps = KsrArchive::new();
    no_gaps.replace_unique(
        "Metadata/project_settings.config",
        "\"gap_infill_speed\": \"250\"",
        "\"gap_infill_speed\": \"0\"",
    );
    let no_gaps = first_record(no_gaps.bytes());
    assert_eq!(baseline.0, no_gaps.0);
    assert!(baseline.1);
    assert!(!no_gaps.1);

    let mut target = KsrArchive::new();
    target.replace_unique(
        "Metadata/project_settings.config",
        "\"gap_fill_target\": \"nowhere\"",
        "\"gap_fill_target\": \"everywhere\"",
    );
    assert_eq!(baseline, first_record(target.bytes()));
}

#[test]
fn task22o1_fixture_arc_fitting_selects_one_fifth_resolution() {
    let arc = first_record_resolution(ksr_project());
    let mut no_arc = KsrArchive::new();
    no_arc.replace_unique(
        "Metadata/project_settings.config",
        "\"enable_arc_fitting\": \"1\"",
        "\"enable_arc_fitting\": \"0\"",
    );
    let no_arc = first_record_resolution(no_arc.bytes());

    assert_eq!(arc, 2_400.0);
    assert_eq!(no_arc, 12_000.0);
    assert_eq!(arc, 0.2 * no_arc);
}
