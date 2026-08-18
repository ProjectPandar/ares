use crate::{
    geometry::Polygon,
    project_slice::{
        prepare_infill::vertical_shell_projection::{self, GeometryStep},
        tests::{
            prepare_infill::vertical_shells::ksr::{cache_totals, successor_checksum_parts},
            support::KsrArchive,
        },
    },
};

use super::fixture;

const PARENT_MARKER: i128 = 0x4f32_305f_5041_5245_4e54;

#[test]
fn task22o20_ksr_projection_is_repeatable() {
    vertical_shell_projection::reset_geometry_hooks();
    let first = fixture::prepare(KsrArchive::new().bytes());
    let first_capture = (
        o19_checksum_from_o20(&first),
        cache_totals(&first.caches),
        event_totals(),
        projection_totals(&first.projections),
        successor_checksum(&first),
    );

    vertical_shell_projection::reset_geometry_hooks();
    let second = fixture::prepare(KsrArchive::new().bytes());
    let second_capture = (
        o19_checksum_from_o20(&second),
        cache_totals(&second.caches),
        event_totals(),
        projection_totals(&second.projections),
        successor_checksum(&second),
    );
    assert_eq!(second_capture, first_capture);
    vertical_shell_projection::reset_geometry_hooks();
}

pub(super) fn projection_digest(
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

fn successor_checksum(
    prepared: &vertical_shell_projection::PreparedPostVerticalShellProjection,
) -> i128 {
    let mut checksum = PARENT_MARKER;
    mix(&mut checksum, o19_checksum_from_o20(prepared));
    mix(&mut checksum, projection_digest(&prepared.projections));
    checksum
}

fn o19_checksum_from_o20(
    prepared: &vertical_shell_projection::PreparedPostVerticalShellProjection,
) -> i128 {
    successor_checksum_parts(&prepared.predecessor, &prepared.objects, &prepared.caches)
}

fn projection_totals(
    objects: &[vertical_shell_projection::types::VerticalShellProjectionObject],
) -> [usize; 8] {
    let mut totals = [objects.len(), 0, 0, 0, 0, 0, 0, 0];
    for object in objects {
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
            totals[6] += projection
                .shell
                .iter()
                .map(|path| path.points().len())
                .sum::<usize>();
            totals[7] += projection
                .holes
                .iter()
                .map(|path| path.points().len())
                .sum::<usize>();
        }
    }
    totals
}

fn event_totals() -> [usize; 8] {
    let mut totals = [0; 8];
    for event in vertical_shell_projection::geometry_events() {
        totals[match event {
            GeometryStep::TopVisit => 0,
            GeometryStep::BottomVisit => 1,
            GeometryStep::HoleIntersection => 2,
            GeometryStep::ShellUnion => 3,
            GeometryStep::TopAnchorOffset => 4,
            GeometryStep::TopAnchorIntersection => 5,
            GeometryStep::BottomAnchorOffset => 6,
            GeometryStep::BottomAnchorIntersection => 7,
        }] += 1;
    }
    totals
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
