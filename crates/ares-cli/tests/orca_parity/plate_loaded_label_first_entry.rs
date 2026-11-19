//! `bbs_3mf.cpp:4986-5004,2466-2499`: one loaded label assignment per plate object.

use crate::runner::native_stream_case;

const UPSTREAM: &[&str] = &[
    "Format/bbs_3mf.cpp:4986-5004::_handle_end_config_plater_instance",
    "Format/bbs_3mf.cpp:2466-2499",
    "Model.hpp:1437-1443::ModelObject::get_instance_label_id",
];
const SAME_POSE: &[u8] =
    include_bytes!("../../../../tests/parity/plate-loaded-label-first-entry/x1c-same-pose.3mf");

#[test]
fn first_plate_entry_labels_only_its_instance() {
    check_x1c(
        "x1c-same-pose",
        SAME_POSE,
        "2c41554721d8780fb58f6c1cfcd0062bc60995c48a977334ca04e4a9d86cc9be",
        "9,20",
    );
}

#[test]
fn first_plate_entry_labels_only_its_instance_after_saved_transforms() {
    check_x1c(
        "x1c-rotated-scaled",
        include_bytes!(
            "../../../../tests/parity/plate-loaded-label-first-entry/x1c-rotated-scaled.3mf"
        ),
        "607ff272a336ac8ce603111e45de78af9fb283312d95d93d06b415d44df7c628",
        "9,20",
    );
}

#[test]
fn first_plate_entry_is_chosen_in_xml_order() {
    check_x1c(
        "x1c-reversed-entries",
        include_bytes!(
            "../../../../tests/parity/plate-loaded-label-first-entry/x1c-reversed-entries.3mf"
        ),
        "556f54891ac123f19767554337ceb4036093d2e8dabfb7e924d91b0e2b49b60c",
        "8,21",
    );
}

#[test]
fn first_plate_entry_with_zero_id_still_precedes_later_entries() {
    check_x1c(
        "x1c-first-entry-zero",
        include_bytes!(
            "../../../../tests/parity/plate-loaded-label-first-entry/x1c-first-entry-zero.3mf"
        ),
        "ea3903c26e83446a78c7e492e6754ca2e634ef666bc2872d10bdfd655061e864",
        "8,9",
    );
}

#[test]
fn first_plate_entry_preserves_both_k1_exclude_object_definitions() {
    native_stream_case::check(
        ("plate-loaded-label-first-entry", UPSTREAM),
        "k1-rotated-scaled",
        include_bytes!(
            "../../../../tests/parity/plate-loaded-label-first-entry/k1-rotated-scaled.3mf"
        ),
        "439167e02d2fe39392d78b1d8e024c0e103c098dfafe02c3544818a35b93026c",
        (false, |reference, _| {
            assert_eq!(
                reference
                    .native
                    .matches("EXCLUDE_OBJECT_DEFINE NAME=")
                    .count(),
                2
            );
            for name in ["Probe_id_0_copy_0", "Probe_id_1_copy_0"] {
                assert!(
                    reference
                        .native
                        .contains(&format!("EXCLUDE_OBJECT_DEFINE NAME={name} "))
                );
            }
        }),
    );
}

#[test]
fn first_plate_entry_preserves_all_imported_plate_instances() {
    let project = ares_core::load_project(SAME_POSE).unwrap();
    assert_eq!(
        project.plates()[0].instances(),
        &[[101, 0, 20], [101, 1, 21]]
    );
    let instances = project.objects()[0].instances();
    assert_eq!(instances.len(), 2);
    assert_eq!(instances[0].loaded_label_id(), Some(20));
    assert_eq!(instances[1].loaded_label_id(), None);
}

fn check_x1c(case: &str, project: &[u8], sha: &str, expected_label_ids: &str) {
    native_stream_case::check(
        ("plate-loaded-label-first-entry", UPSTREAM),
        case,
        project,
        sha,
        (false, |reference, _| {
            assert_eq!(
                reference
                    .native
                    .lines()
                    .find_map(|line| line.strip_prefix("; model label id: ")),
                Some(expected_label_ids),
            );
            assert!(
                reference
                    .native
                    .lines()
                    .any(|line| line.starts_with("M624 "))
            );
        }),
    );
}
