use crate::{
    geometry::ExPolygon,
    project_slice::{prepare_infill::vertical_shell_filtering, tests::support::KsrArchive},
};

#[test]
fn task22o23_zip_repack_and_non_slicing_rename_preserve_filtering() {
    let baseline = super::fixture::prepare(KsrArchive::new().bytes());
    let baseline_digest = digest(&baseline);
    vertical_shell_filtering::dispose(baseline);

    let mut renamed = KsrArchive::new();
    renamed.replace(
        "Metadata/model_settings.config",
        "value=\"ksr_fdmtest_v4.drc\"",
        "value=\"task22o23_renamed\"",
    );
    let renamed = super::fixture::prepare(renamed.bytes_stored_reverse());
    assert_eq!(digest(&renamed), baseline_digest);
    vertical_shell_filtering::dispose(renamed);

    let timestamped = super::fixture::prepare(KsrArchive::new().bytes_with_timestamp());
    assert_eq!(digest(&timestamped), baseline_digest);
    vertical_shell_filtering::dispose(timestamped);
}

#[test]
fn task22o23_component_transform_scaling_remains_source_derived() {
    let baseline = super::fixture::prepare(KsrArchive::new().bytes());
    let baseline_digest = digest(&baseline);
    vertical_shell_filtering::dispose(baseline);
    let mut scaled = KsrArchive::new();
    scaled.replace_unique(
        "3D/3dmodel.model",
        "transform=\"1 0 0 0 1 0 0 0 1 0 0 0\"",
        "transform=\"2 0 0 0 1 0 0 0 1 0 0 0\"",
    );
    let scaled = super::fixture::prepare(scaled.bytes());
    assert_ne!(digest(&scaled), baseline_digest);
    vertical_shell_filtering::dispose(scaled);
}

fn digest(output: &vertical_shell_filtering::PreparedPostVerticalShellFiltering) -> i128 {
    let mut digest = 0x4f23_i128;
    for object in &output.filters {
        mix(&mut digest, object.records.len() as i128);
        for record in &object.records {
            match record {
                None => mix(&mut digest, -1),
                Some(record) => expolygons(&mut digest, &record.filtered_shell),
            }
        }
    }
    digest
}

fn expolygons(digest: &mut i128, expolygons: &[ExPolygon]) {
    mix(digest, expolygons.len() as i128);
    for expolygon in expolygons {
        for path in std::iter::once(expolygon.contour()).chain(expolygon.holes()) {
            mix(digest, path.points().len() as i128);
            for point in path.points() {
                mix(digest, point.x() as i128);
                mix(digest, point.y() as i128);
            }
        }
    }
}

fn mix(digest: &mut i128, value: i128) {
    *digest = digest.wrapping_mul(0x100_0000_01b3).wrapping_add(value);
}
