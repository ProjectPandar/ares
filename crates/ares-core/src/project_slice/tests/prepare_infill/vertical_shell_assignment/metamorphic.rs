use crate::project_slice::{prepare_infill::vertical_shell_assignment, tests::support::KsrArchive};

#[test]
fn task22o24_zip_repack_timestamp_and_non_slicing_rename_preserve_assignment() {
    let baseline = super::fixture::prepare(KsrArchive::new().bytes());
    let baseline_digest = digest(&baseline);
    vertical_shell_assignment::dispose(baseline);

    let mut renamed = KsrArchive::new();
    renamed.replace(
        "Metadata/model_settings.config",
        "value=\"ksr_fdmtest_v4.drc\"",
        "value=\"task22o24_renamed\"",
    );
    let renamed = super::fixture::prepare(renamed.bytes_stored_reverse());
    assert_eq!(digest(&renamed), baseline_digest);
    vertical_shell_assignment::dispose(renamed);

    let timestamped = super::fixture::prepare(KsrArchive::new().bytes_with_timestamp());
    assert_eq!(digest(&timestamped), baseline_digest);
    vertical_shell_assignment::dispose(timestamped);
}

#[test]
fn task22o24_component_transform_changes_only_predecessor_derived_geometry() {
    let baseline = super::fixture::prepare(KsrArchive::new().bytes());
    let baseline_digest = digest(&baseline);
    vertical_shell_assignment::dispose(baseline);
    let mut scaled = KsrArchive::new();
    scaled.replace_unique(
        "3D/3dmodel.model",
        "transform=\"1 0 0 0 1 0 0 0 1 0 0 0\"",
        "transform=\"2 0 0 0 1 0 0 0 1 0 0 0\"",
    );
    let scaled = super::fixture::prepare(scaled.bytes());
    assert_ne!(digest(&scaled), baseline_digest);
    vertical_shell_assignment::dispose(scaled);
}

fn digest(output: &vertical_shell_assignment::PreparedPostVerticalShellAssignment) -> i128 {
    let mut digest = 0x4f24_i128;
    for object in &output.objects {
        for record in &object.records {
            let Some(record) = record else {
                mix(&mut digest, -1);
                continue;
            };
            for surface in &record.fill_surfaces {
                surface_digest(&mut digest, surface);
            }
        }
    }
    digest
}

fn surface_digest(digest: &mut i128, surface: &crate::project_slice::region_slices::RegionSurface) {
    let (kind, expolygon, thickness, layers, angle, extra) = surface.as_parts();
    mix(digest, kind as i128);
    mix(digest, thickness.to_bits() as i128);
    mix(digest, layers as i128);
    mix(digest, angle.to_bits() as i128);
    mix(digest, extra as i128);
    for path in std::iter::once(expolygon.contour()).chain(expolygon.holes()) {
        for point in path.points() {
            mix(digest, point.x() as i128);
            mix(digest, point.y() as i128);
        }
    }
}

fn mix(digest: &mut i128, value: i128) {
    *digest = digest
        .wrapping_mul(0x1000003d)
        .wrapping_add(value)
        .rotate_left(11);
}
