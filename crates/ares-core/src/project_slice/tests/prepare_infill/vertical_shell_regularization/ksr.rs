use crate::{
    geometry::Polygon,
    project_slice::{
        prepare_infill::{
            vertical_shell_projection, vertical_shell_regularization, vertical_shell_trimming,
        },
        tests::{
            prepare_infill::vertical_shells::ksr::{cache_totals, successor_checksum_parts},
            support::KsrArchive,
        },
    },
};

use super::{
    fixture,
    metamorphic::{mix, regularization_digest},
};

const O19_CHECKSUM: i128 = 148_296_943_860_974_241_781_127_169_756_103_364_063;
const O19_TOTALS: [usize; 9] = [1, 460, 0, 460, 572, 713, 1_227, 60_370, 2_512];
const O20_CHECKSUM: i128 = -106_767_561_006_193_260_948_265_111_057_697_183_253;
const O20_TOTALS: [usize; 8] = [1, 460, 0, 460, 1_688, 1_224, 36_512, 69_033];
const O20_EVENTS: [usize; 8] = [1_830, 917, 1_539, 749, 0, 0, 0, 0];
const O21_CHECKSUM: i128 = -86_220_837_291_247_746_226_319_093_859_583_939_318;
const O21_TOTALS: [usize; 6] = [1, 460, 0, 460, 7_704, 104_680];
const O21_EVENTS: [usize; 5] = [460, 460, 460, 460, 259];
const O22_CHECKSUM: i128 = 134_936_948_052_282_121_922_360_252_649_864_225_707;
const O22_TOTALS: [usize; 8] = [1, 460, 0, 460, 632, 632, 128, 34_557];
const O22_EVENTS: [usize; 4] = [259, 259, 259, 259];
const O22_RADII_DIGEST: i128 = -119_839_535_044_106_185_061_007_902_266_478_724_784;

#[test]
fn task22o22_ksr_regularization_is_parent_guarded_and_repeatable() {
    let first = capture();
    assert_eq!(
        first,
        (O22_CHECKSUM, O22_TOTALS, O22_EVENTS, O22_RADII_DIGEST)
    );
    let second = capture();
    assert_eq!(
        second,
        (O22_CHECKSUM, O22_TOTALS, O22_EVENTS, O22_RADII_DIGEST)
    );
}

fn capture() -> (i128, [usize; 8], [usize; 4], i128) {
    vertical_shell_projection::reset_geometry_hooks();
    vertical_shell_trimming::reset_geometry_hooks();
    vertical_shell_regularization::reset_geometry_hooks();
    let output = fixture::prepare(KsrArchive::new().bytes());
    assert_eq!(
        successor_checksum_parts(&output.predecessor, &output.objects, &output.caches),
        O19_CHECKSUM
    );
    assert_eq!(cache_totals(&output.caches), O19_TOTALS);
    assert_eq!(o20_checksum(&output), O20_CHECKSUM);
    assert_eq!(o20_totals(&output), O20_TOTALS);
    assert_eq!(o20_events(), O20_EVENTS);
    assert_eq!(o21_checksum(&output), O21_CHECKSUM);
    assert_eq!(o21_totals(&output), O21_TOTALS);
    assert_eq!(o21_events(), O21_EVENTS);
    (
        o22_checksum(&output),
        o22_totals(&output),
        o22_events(),
        radii_digest(&output),
    )
}

fn o20_checksum(
    output: &vertical_shell_regularization::PreparedPostVerticalShellRegularization,
) -> i128 {
    let mut checksum = 0x4f32_305f_5041_5245_4e54_i128;
    mix(
        &mut checksum,
        successor_checksum_parts(&output.predecessor, &output.objects, &output.caches),
    );
    mix(&mut checksum, projection_digest(&output.projections));
    checksum
}

fn o21_checksum(
    output: &vertical_shell_regularization::PreparedPostVerticalShellRegularization,
) -> i128 {
    let mut checksum = 0x4f32_315f_5041_5245_4e54_i128;
    mix(&mut checksum, o20_checksum(output));
    mix(&mut checksum, trim_digest(&output.trims));
    checksum
}

fn o22_checksum(
    output: &vertical_shell_regularization::PreparedPostVerticalShellRegularization,
) -> i128 {
    let mut checksum = 0x4f32_325f_5041_5245_4e54_i128;
    mix(&mut checksum, o21_checksum(output));
    mix(
        &mut checksum,
        regularization_digest(&output.regularizations),
    );
    checksum
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

fn o20_totals(
    output: &vertical_shell_regularization::PreparedPostVerticalShellRegularization,
) -> [usize; 8] {
    let mut totals = [output.projections.len(), 0, 0, 0, 0, 0, 0, 0];
    for object in &output.projections {
        totals[1] += object.records.len();
        totals[2] += object
            .records
            .iter()
            .filter(|record| record.is_none())
            .count();
        for projection in object.records.iter().flatten() {
            totals[3] += 1;
            totals[4] += projection.shell.len();
            totals[5] += projection.holes.len();
            totals[6] += point_count(&projection.shell);
            totals[7] += point_count(&projection.holes);
        }
    }
    totals
}

fn o21_totals(
    output: &vertical_shell_regularization::PreparedPostVerticalShellRegularization,
) -> [usize; 6] {
    let mut totals = [output.trims.len(), 0, 0, 0, 0, 0];
    for object in &output.trims {
        totals[1] += object.records.len();
        totals[2] += object
            .records
            .iter()
            .filter(|record| record.is_none())
            .count();
        for trim in object.records.iter().flatten() {
            totals[3] += 1;
            totals[4] += trim.shell.len();
            totals[5] += point_count(&trim.shell);
        }
    }
    totals
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

fn o20_events() -> [usize; 8] {
    let mut totals = [0; 8];
    for event in vertical_shell_projection::geometry_events() {
        totals[event as usize] += 1;
    }
    totals
}

fn o21_events() -> [usize; 5] {
    let mut totals = [0; 5];
    for event in vertical_shell_trimming::geometry_events() {
        totals[event as usize] += 1;
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

fn point_count(paths: &[Polygon]) -> usize {
    paths.iter().map(|path| path.points().len()).sum()
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
