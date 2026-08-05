use crate::{
    geometry::Polygon,
    project_slice::{
        prepare_infill::vertical_shell_regularization::types::{
            VerticalShellRegularization, VerticalShellRegularizationObject,
        },
        tests::support::KsrArchive,
    },
};

use super::fixture;

#[test]
fn task22o22_zip_repack_and_non_slicing_rename_preserve_regularization() {
    let baseline = fixture::prepare(KsrArchive::new().bytes());
    let baseline = regularization_digest(&baseline.regularizations);
    let mut renamed = KsrArchive::new();
    renamed.replace(
        "Metadata/model_settings.config",
        "value=\"ksr_fdmtest_v4.drc\"",
        "value=\"task22o22_renamed\"",
    );
    let renamed = fixture::prepare(renamed.bytes_stored_reverse());
    assert_eq!(regularization_digest(&renamed.regularizations), baseline);
}

#[test]
fn task22o22_component_x_scale_changes_trim_and_regularization_geometry() {
    let baseline = fixture::prepare(KsrArchive::new().bytes());
    let baseline_trim = trim_digest(&baseline.trims);
    let baseline_regularization = regularization_digest(&baseline.regularizations);
    let mut scaled = KsrArchive::new();
    scaled.replace_unique(
        "3D/3dmodel.model",
        "transform=\"1 0 0 0 1 0 0 0 1 0 0 0\"",
        "transform=\"2 0 0 0 1 0 0 0 1 0 0 0\"",
    );
    let scaled = fixture::prepare(scaled.bytes());
    assert_ne!(trim_digest(&scaled.trims), baseline_trim);
    assert_ne!(
        regularization_digest(&scaled.regularizations),
        baseline_regularization
    );
}

pub(super) fn regularization_digest(objects: &[VerticalShellRegularizationObject]) -> i128 {
    let mut digest = 0x4f22_i128;
    mix(&mut digest, objects.len() as i128);
    for object in objects {
        mix(&mut digest, 0x4f424a);
        mix(&mut digest, object.records.len() as i128);
        for record in &object.records {
            match record {
                None => mix(&mut digest, -1),
                Some(regularization) => regularization_record(&mut digest, regularization),
            }
        }
    }
    digest
}

fn regularization_record(digest: &mut i128, regularization: &VerticalShellRegularization) {
    mix(digest, 1);
    mix(digest, regularization.regularized_shell.len() as i128);
    for expolygon in &regularization.regularized_shell {
        path(digest, expolygon.contour());
        mix(digest, expolygon.holes().len() as i128);
        for hole in expolygon.holes() {
            path(digest, hole);
        }
    }
}

fn trim_digest(
    objects: &[crate::project_slice::prepare_infill::vertical_shell_trimming::types::VerticalShellTrimObject],
) -> i128 {
    let mut digest = 0x4f21_i128;
    for object in objects {
        mix(&mut digest, object.records.len() as i128);
        for trim in object.records.iter().flatten() {
            for path in &trim.shell {
                path_points(&mut digest, path);
            }
        }
    }
    digest
}

fn path(digest: &mut i128, polygon: &Polygon) {
    mix(digest, 0x50415448);
    path_points(digest, polygon);
}

fn path_points(digest: &mut i128, polygon: &Polygon) {
    mix(digest, polygon.points().len() as i128);
    for point in polygon.points() {
        mix(digest, point.x() as i128);
        mix(digest, point.y() as i128);
    }
}

pub(super) fn mix(digest: &mut i128, value: i128) {
    *digest = digest.wrapping_mul(0x100_0000_01b3).wrapping_add(value);
}
