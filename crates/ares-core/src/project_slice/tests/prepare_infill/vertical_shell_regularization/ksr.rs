use crate::{
    geometry::Polygon,
    project_slice::{
        perimeters::classic::traversal::PreparedPostClassicTraversal,
        prepare_infill::{
            surface_type_detection::PreparedSurfaceTypeObject,
            vertical_shell_projection::{self, types::VerticalShellProjectionObject},
            vertical_shell_regularization::{self, types::VerticalShellRegularizationObject},
            vertical_shell_trimming::{self, types::VerticalShellTrimObject},
            vertical_shells::types::VerticalShellCacheObject,
        },
        tests::{
            prepare_infill::vertical_shells::ksr::successor_checksum_parts, support::KsrArchive,
        },
    },
};

use super::{
    fixture,
    metamorphic::{mix, regularization_digest},
};

#[test]
fn task22o22_ksr_regularization_is_repeatable() {
    let first = capture();
    let second = capture();
    assert_eq!(second, first);
}

pub(in crate::project_slice::tests::prepare_infill) fn capture()
-> (i128, [usize; 8], [usize; 4], i128) {
    vertical_shell_projection::reset_geometry_hooks();
    vertical_shell_trimming::reset_geometry_hooks();
    vertical_shell_regularization::reset_geometry_hooks();
    let output = fixture::prepare(KsrArchive::new().bytes());
    (
        o22_checksum(&output),
        o22_totals(&output),
        o22_events(),
        radii_digest(&output),
    )
}

fn o22_checksum(
    output: &vertical_shell_regularization::PreparedPostVerticalShellRegularization,
) -> i128 {
    o22_checksum_parts(O22ChecksumParts {
        predecessor: &output.predecessor,
        objects: &output.objects,
        caches: &output.caches,
        projections: &output.projections,
        trims: &output.trims,
        regularizations: &output.regularizations,
    })
}

pub(in crate::project_slice::tests::prepare_infill) struct O22ChecksumParts<'a> {
    pub(in crate::project_slice::tests::prepare_infill) predecessor:
        &'a PreparedPostClassicTraversal,
    pub(in crate::project_slice::tests::prepare_infill) objects: &'a [PreparedSurfaceTypeObject],
    pub(in crate::project_slice::tests::prepare_infill) caches: &'a [VerticalShellCacheObject],
    pub(in crate::project_slice::tests::prepare_infill) projections:
        &'a [VerticalShellProjectionObject],
    pub(in crate::project_slice::tests::prepare_infill) trims: &'a [VerticalShellTrimObject],
    pub(in crate::project_slice::tests::prepare_infill) regularizations:
        &'a [VerticalShellRegularizationObject],
}

pub(in crate::project_slice::tests::prepare_infill) fn o22_checksum_parts(
    parts: O22ChecksumParts<'_>,
) -> i128 {
    let O22ChecksumParts {
        predecessor,
        objects,
        caches,
        projections,
        trims,
        regularizations,
    } = parts;
    let mut o20 = 0x4f32_305f_5041_5245_4e54_i128;
    mix(
        &mut o20,
        successor_checksum_parts(predecessor, objects, caches),
    );
    mix(&mut o20, projection_digest(projections));
    let mut o21 = 0x4f32_315f_5041_5245_4e54_i128;
    mix(&mut o21, o20);
    mix(&mut o21, trim_digest(trims));
    let mut o22 = 0x4f32_325f_5041_5245_4e54_i128;
    mix(&mut o22, o21);
    mix(&mut o22, regularization_digest(regularizations));
    o22
}

fn projection_digest(
    objects: &[vertical_shell_projection::types::VerticalShellProjectionObject],
) -> i128 {
    let mut digest = 0x4f20_i128;
    mix(&mut digest, objects.len() as i128);
    for object in objects {
        mix(&mut digest, 0x4f424a);
        mix(&mut digest, object.records.len() as i128);
        for record in &object.records {
            match record {
                None => mix(&mut digest, -1),
                Some(projection) => {
                    mix(&mut digest, 1);
                    paths(&mut digest, &projection.shell);
                    paths(&mut digest, &projection.holes);
                }
            }
        }
    }
    digest
}

fn trim_digest(objects: &[vertical_shell_trimming::types::VerticalShellTrimObject]) -> i128 {
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
                    trim_paths(&mut digest, &trim.shell);
                }
            }
        }
    }
    digest
}

fn o22_totals(
    output: &vertical_shell_regularization::PreparedPostVerticalShellRegularization,
) -> [usize; 8] {
    let mut totals = [output.regularizations.len(), 0, 0, 0, 0, 0, 0, 0];
    for object in &output.regularizations {
        totals[1] += object.records.len();
        totals[2] += object
            .records
            .iter()
            .filter(|record| record.is_none())
            .count();
        for record in object.records.iter().flatten() {
            totals[3] += 1;
            totals[4] += record.regularized_shell.len();
            totals[5] += record.regularized_shell.len();
            for expolygon in &record.regularized_shell {
                totals[6] += expolygon.holes().len();
                totals[7] += expolygon.contour().points().len()
                    + expolygon
                        .holes()
                        .iter()
                        .map(|hole| hole.points().len())
                        .sum::<usize>();
            }
        }
    }
    totals
}

fn o22_events() -> [usize; 4] {
    let mut totals = [0; 4];
    for event in vertical_shell_regularization::geometry_events() {
        totals[event as usize] += 1;
    }
    totals
}

fn radii_digest(
    output: &vertical_shell_regularization::PreparedPostVerticalShellRegularization,
) -> i128 {
    let mut digest = 0x5241444949_i128;
    for traversal in &output.predecessor.objects {
        let records = &traversal
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .records;
        for record in records {
            match record.as_ref().map(|record| record.solid_infill_spacing) {
                None => mix(&mut digest, -1),
                Some(spacing) => mix_radii(&mut digest, spacing),
            }
        }
    }
    digest
}

fn mix_radii(digest: &mut i128, spacing: i64) {
    mix(digest, spacing as i128);
    for bits in vertical_shell_regularization::radii_bits(spacing) {
        mix(digest, bits as i128);
    }
}

fn paths(digest: &mut i128, paths: &[Polygon]) {
    mix(digest, paths.len() as i128);
    for path in paths {
        mix(digest, path.points().len() as i128);
        for point in path.points() {
            mix(digest, point.x() as i128);
            mix(digest, point.y() as i128);
        }
    }
}

fn trim_paths(digest: &mut i128, paths: &[Polygon]) {
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
