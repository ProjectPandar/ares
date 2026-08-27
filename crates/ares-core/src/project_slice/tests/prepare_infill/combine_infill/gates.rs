use crate::project_slice::{
    prepare_infill::{bridge_over_infill::transaction, combine_infill},
    tests::support::KsrArchive,
};

const PROJECT_SETTINGS: &str = "Metadata/project_settings.config";
const MODEL_SETTINGS: &str = "Metadata/model_settings.config";

#[test]
fn archive_combination_with_nonzero_density_produces_successor() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        PROJECT_SETTINGS,
        "\"infill_combination\": \"0\"",
        "\"infill_combination\": \"1\"",
    );

    assert_active_combination(archive, true, &[(true, 15.0)]);
}

#[test]
fn effective_object_and_part_overrides_drive_combination() {
    for archive in [object_override_archive(), part_override_archive()] {
        assert_active_combination(archive, false, &[(true, 15.0)]);
    }
}

#[test]
fn task22o72_materialized_part_combination_at_zero_density_is_an_identity() {
    let archive = part_zero_density_override_archive();
    combine_infill::reset_hooks();
    transaction::reset_hooks();
    let input = super::prepare_o71(archive);
    assert_eq!(
        super::materialized_combination_options(&input),
        [(true, 0.0)]
    );
    assert!(
        !input
            .predecessor
            .predecessor
            .predecessor
            .resolved
            .views
            .full
            .process
            .region
            .infill_combination
            .0
    );
    assert_eq!(
        input
            .predecessor
            .predecessor
            .predecessor
            .resolved
            .views
            .full
            .process
            .region
            .sparse_infill_density
            .0,
        15.0
    );
    let deep_predecessor = std::ptr::from_ref(input.predecessor.predecessor.predecessor.as_ref());
    let before = super::snapshot(&input);

    let output = combine_infill::prepare(input).unwrap();

    assert_eq!(
        std::ptr::from_ref(
            output
                .predecessor
                .predecessor
                .predecessor
                .predecessor
                .as_ref(),
        ),
        deep_predecessor
    );
    assert_eq!(super::snapshot(&output.predecessor).bytes, before.bytes);
    assert_eq!(combine_infill::invocations(), 1);
    assert_eq!(combine_infill::disposals(), 0);
    assert_eq!(transaction::invocations(), 1);
    assert_eq!(transaction::disposals(), 0);

    combine_infill::dispose(output);
    assert_eq!(combine_infill::disposals(), 1);
    assert_eq!(transaction::disposals(), 1);
    combine_infill::reset_hooks();
    transaction::reset_hooks();
}

#[test]
fn later_object_active_combination_is_not_skipped() {
    assert_active_combination(
        later_object_override_archive(),
        false,
        &[(false, 15.0), (true, 15.0)],
    );
}

#[test]
fn task22o72_nonzero_density_sign_does_not_replace_exact_zero() {
    for density in [-f64::MIN_POSITIVE, f64::MIN_POSITIVE] {
        combine_infill::reset_hooks();
        transaction::reset_hooks();
        let mut input = super::prepare_o71(KsrArchive::new());
        let traversal = &mut input.predecessor.predecessor.predecessor;
        let prelude = &mut traversal.objects[0]
            .predecessor
            .predecessor
            .predecessor
            .predecessor;
        let (post_regions, _) = prelude.object.object.as_parts_mut();
        post_regions.regions[0].options.infill_combination.0 = true;
        post_regions.regions[0].options.sparse_infill_density.0 = density;

        let output = combine_infill::prepare(input).unwrap();

        assert_eq!(transaction::disposals(), 0);
        assert_eq!(combine_infill::disposals(), 0);
        combine_infill::dispose(output);
        assert_eq!(transaction::disposals(), 1);
        assert_eq!(combine_infill::disposals(), 1);
        combine_infill::reset_hooks();
        transaction::reset_hooks();
    }
}

fn assert_active_combination(
    archive: KsrArchive,
    expected_global: bool,
    expected_options: &[(bool, f64)],
) {
    combine_infill::reset_hooks();
    transaction::reset_hooks();
    let input = super::prepare_o71(archive);
    assert_eq!(
        super::materialized_combination_options(&input),
        expected_options
    );
    assert_eq!(
        input
            .predecessor
            .predecessor
            .predecessor
            .resolved
            .views
            .full
            .process
            .region
            .infill_combination
            .0,
        expected_global
    );

    let output = combine_infill::prepare(input).unwrap();

    assert_eq!(transaction::invocations(), 1);
    assert_eq!(transaction::disposals(), 0);
    assert_eq!(combine_infill::invocations(), 1);
    assert_eq!(combine_infill::disposals(), 0);
    combine_infill::dispose(output);
    assert_eq!(transaction::disposals(), 1);
    assert_eq!(combine_infill::disposals(), 1);
    combine_infill::reset_hooks();
    transaction::reset_hooks();
}

fn object_override_archive() -> KsrArchive {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        MODEL_SETTINGS,
        concat!(
            "    <metadata key=\"extruder\" value=\"1\"/>\n",
            "    <part id=\"1\" subtype=\"normal_part\">",
        ),
        concat!(
            "    <metadata key=\"extruder\" value=\"1\"/>\n",
            "    <metadata key=\"infill_combination\" value=\"1\"/>\n",
            "    <part id=\"1\" subtype=\"normal_part\">",
        ),
    );
    archive
}

fn part_override_archive() -> KsrArchive {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        MODEL_SETTINGS,
        concat!(
            "    <part id=\"1\" subtype=\"normal_part\">\n",
            "      <metadata key=\"name\" value=\"ksr_fdmtest_v4.drc\"/>",
        ),
        concat!(
            "    <part id=\"1\" subtype=\"normal_part\">\n",
            "      <metadata key=\"infill_combination\" value=\"1\"/>\n",
            "      <metadata key=\"name\" value=\"ksr_fdmtest_v4.drc\"/>",
        ),
    );
    archive
}

fn part_zero_density_override_archive() -> KsrArchive {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        MODEL_SETTINGS,
        concat!(
            "    <part id=\"1\" subtype=\"normal_part\">\n",
            "      <metadata key=\"name\" value=\"ksr_fdmtest_v4.drc\"/>",
        ),
        concat!(
            "    <part id=\"1\" subtype=\"normal_part\">\n",
            "      <metadata key=\"infill_combination\" value=\"1\"/>\n",
            "      <metadata key=\"sparse_infill_density\" value=\"0%\"/>\n",
            "      <metadata key=\"name\" value=\"ksr_fdmtest_v4.drc\"/>",
        ),
    );
    archive
}

fn later_object_override_archive() -> KsrArchive {
    let mut archive = super::super::bridge_over_infill::multi_object::two_object_archive();
    archive.replace_unique(
        MODEL_SETTINGS,
        concat!(
            "  <object id=\"3\">\n",
            "    <metadata key=\"name\" value=\"ksr_fdmtest_v4-copy.drc\"/>",
        ),
        concat!(
            "  <object id=\"3\">\n",
            "    <metadata key=\"infill_combination\" value=\"1\"/>\n",
            "    <metadata key=\"name\" value=\"ksr_fdmtest_v4-copy.drc\"/>",
        ),
    );
    archive
}
