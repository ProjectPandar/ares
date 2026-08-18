use crate::{
    geometry::Polygon,
    project_slice::{
        prepare_infill::{vertical_shell_projection, vertical_shell_trimming},
        tests::{
            prepare_infill::vertical_shells::ksr::successor_checksum_parts, support::KsrArchive,
        },
    },
};

use super::{fixture, metamorphic::trim_digest};

#[test]
fn task22o21_ksr_trim_is_repeatable() {
    let first = capture();
    let second = capture();
    assert_eq!(second, first);
}

fn capture() -> (i128, [usize; 6], [usize; 5]) {
    vertical_shell_projection::reset_geometry_hooks();
    vertical_shell_trimming::reset_geometry_hooks();
    let output = fixture::prepare(KsrArchive::new().bytes());
    (o21_checksum(&output), o21_totals(&output), o21_events())
}

fn o20_checksum(output: &vertical_shell_trimming::PreparedPostVerticalShellTrim) -> i128 {
    let mut checksum = 0x4f32_305f_5041_5245_4e54_i128;
    mix(
        &mut checksum,
        successor_checksum_parts(&output.predecessor, &output.objects, &output.caches),
    );
    mix(&mut checksum, projection_digest(&output.projections));
    checksum
}

fn o21_checksum(output: &vertical_shell_trimming::PreparedPostVerticalShellTrim) -> i128 {
    let mut checksum = 0x4f32_315f_5041_5245_4e54_i128;
    mix(&mut checksum, o20_checksum(output));
    mix(&mut checksum, trim_digest(&output.trims));
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

fn o21_totals(output: &vertical_shell_trimming::PreparedPostVerticalShellTrim) -> [usize; 6] {
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

fn o21_events() -> [usize; 5] {
    let mut totals = [0; 5];
    for event in vertical_shell_trimming::geometry_events() {
        totals[match event {
            vertical_shell_trimming::GeometryStep::SafetyOffset => 0,
            vertical_shell_trimming::GeometryStep::SafetyIntersection => 1,
            vertical_shell_trimming::GeometryStep::Difference => 2,
            vertical_shell_trimming::GeometryStep::EmptyGate => 3,
            vertical_shell_trimming::GeometryStep::SolidAppend => 4,
        }] += 1;
    }
    totals
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

fn mix(digest: &mut i128, value: i128) {
    *digest = digest.wrapping_mul(0x100_0000_01b3).wrapping_add(value);
}
