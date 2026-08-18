use super::ksr::digest::{
    event_sequence_digest, propagation_event_counts, surface_sequence_digest,
};
use crate::project_slice::{
    prepare_infill::horizontal_shell_propagation, tests::support::KsrArchive,
};

fn capture(bytes: Vec<u8>) -> ([usize; 7], usize, usize, i128, i128) {
    horizontal_shell_propagation::reset_hooks();
    let output = super::fixture::prepare(bytes);
    let events = horizontal_shell_propagation::events();
    let capture = (
        propagation_event_counts(&events),
        horizontal_shell_propagation::geometry_events().len(),
        horizontal_shell_propagation::commits(),
        event_sequence_digest(&events),
        surface_sequence_digest(&output.objects),
    );
    horizontal_shell_propagation::dispose(output);
    capture
}

#[test]
fn task22o26_active_archive_is_invariant_to_zip_order_compression_and_timestamp() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\t\"ensure_vertical_shell_thickness\": \"ensure_all\",",
        "\t\"ensure_vertical_shell_thickness\": \"ensure_moderate\",",
    );
    let reverse = archive.clone().bytes_stored_reverse();
    let timestamp = archive.clone().bytes_with_timestamp();
    let mut renamed = archive.clone();
    renamed.replace(
        "Metadata/model_settings.config",
        "value=\"ksr_fdmtest_v4.drc\"",
        "value=\"task22o26_renamed\"",
    );
    let renamed = renamed.bytes_stored_reverse();
    let expected = capture(archive.bytes());
    assert_eq!(capture(reverse), expected);
    assert_eq!(capture(timestamp), expected);
    assert_eq!(capture(renamed), expected);
}
