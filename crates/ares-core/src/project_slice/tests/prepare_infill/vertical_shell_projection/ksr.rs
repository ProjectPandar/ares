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

const O19_SUCCESSOR_CHECKSUM: i128 = 148_296_943_860_974_241_781_127_169_756_103_364_063;
const O19_TOTALS: [usize; 9] = [1, 460, 0, 460, 572, 713, 1_227, 60_370, 2_512];
const PARENT_MARKER: i128 = 0x4f32_305f_5041_5245_4e54;
const O20_SUCCESSOR_CHECKSUM: i128 = -106_767_561_006_193_260_948_265_111_057_697_183_253;
const O20_TOTALS: [usize; 8] = [1, 460, 0, 460, 1_688, 1_224, 36_512, 69_033];
const O20_EVENTS: [usize; 8] = [1_830, 917, 1_539, 749, 0, 0, 0, 0];

#[test]
fn task22o20_ksr_projection_is_parent_guarded_and_repeatable() {
    vertical_shell_projection::reset_geometry_hooks();
    let first = fixture::prepare(KsrArchive::new().bytes());
    assert_eq!(o19_checksum_from_o20(&first), O19_SUCCESSOR_CHECKSUM);
    assert_eq!(cache_totals(&first.caches), O19_TOTALS);
    let first_events = event_totals();
    assert_eq!(first_events, O20_EVENTS);
    let first_totals = projection_totals(&first.projections);
    assert_eq!(first_totals, O20_TOTALS);
    let first_checksum = successor_checksum(&first);
    assert_eq!(first_checksum, O20_SUCCESSOR_CHECKSUM);

    vertical_shell_projection::reset_geometry_hooks();
    let second = fixture::prepare(KsrArchive::new().bytes());
    assert_eq!(o19_checksum_from_o20(&second), O19_SUCCESSOR_CHECKSUM);
    assert_eq!(cache_totals(&second.caches), O19_TOTALS);
    assert_eq!(event_totals(), O20_EVENTS);
    assert_eq!(projection_totals(&second.projections), O20_TOTALS);
    assert_eq!(successor_checksum(&second), O20_SUCCESSOR_CHECKSUM);
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
