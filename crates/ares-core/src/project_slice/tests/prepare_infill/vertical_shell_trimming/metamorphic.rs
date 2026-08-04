use crate::{
    geometry::Polygon,
    project_slice::{
        prepare_infill::vertical_shell_trimming::types::VerticalShellTrimObject,
        tests::support::KsrArchive,
    },
};

use super::fixture;

#[test]
fn task22o21_zip_repack_and_non_slicing_rename_preserve_trims() {
    let baseline = fixture::prepare(KsrArchive::new().bytes());
    let baseline = trim_digest(&baseline.trims);
    let mut renamed = KsrArchive::new();
    renamed.replace(
        "Metadata/model_settings.config",
        "value=\"ksr_fdmtest_v4.drc\"",
        "value=\"task22o21_renamed\"",
    );
    let renamed = fixture::prepare(renamed.bytes_stored_reverse());
    assert_eq!(trim_digest(&renamed.trims), baseline);
}

#[test]
fn task22o21_component_x_scale_changes_source_and_trim_geometry() {
    let baseline = fixture::prepare(KsrArchive::new().bytes());
    let baseline_digest = trim_digest(&baseline.trims);
    let mut scaled = KsrArchive::new();
    scaled.replace_unique(
        "3D/3dmodel.model",
        "transform=\"1 0 0 0 1 0 0 0 1 0 0 0\"",
        "transform=\"2 0 0 0 1 0 0 0 1 0 0 0\"",
    );
    let scaled = fixture::prepare(scaled.bytes());
    assert_ne!(trim_digest(&scaled.trims), baseline_digest);
}

pub(super) fn trim_digest(objects: &[VerticalShellTrimObject]) -> i128 {
    let mut digest = 0x4f21_i128;
    mix(&mut digest, objects.len() as i128);
    for object in objects {
        mix(&mut digest, 0x4f424a);
        mix(&mut digest, object.records.len() as i128);
        for record in &object.records {
            match record {
                None => mix(&mut digest, -1),
                Some(trim) => {
                    mix(&mut digest, 1);
                    paths(&mut digest, &trim.shell);
                }
            }
        }
    }
    digest
}

fn paths(digest: &mut i128, paths: &[Polygon]) {
    mix(digest, paths.len() as i128);
    for path in paths {
        mix(digest, 0x50415448);
        mix(digest, path.points().len() as i128);
        for point in path.points() {
            mix(digest, 0x504f494e54);
            mix(digest, point.x() as i128);
            mix(digest, point.y() as i128);
        }
    }
}

fn mix(digest: &mut i128, value: i128) {
    *digest = digest.wrapping_mul(0x100_0000_01b3).wrapping_add(value);
}
