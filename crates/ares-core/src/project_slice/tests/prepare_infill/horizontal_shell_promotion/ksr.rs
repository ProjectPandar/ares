pub(in crate::project_slice::tests::prepare_infill) mod digest;

use crate::project_slice::{
    prepare_infill::horizontal_shell_promotion, region_slices::RegionSurfaceKind,
    tests::support::KsrArchive,
};
use digest::{
    event_sequence_digest, mix, record_digests, record_positions, record_sequence_digest,
    surfaces_digest,
};

#[test]
fn task22o25_ksr_empty_schedule_is_repeatable_and_preserves_surfaces() {
    let first = capture();
    let second = capture();
    assert_eq!(second, first);
    assert_eq!(first.unchanged_records, first.records);
    assert_eq!(first.before_kinds, first.after_kinds);
    assert_eq!(first.before_geometry, first.after_geometry);
    assert_eq!(first.before_surface_digest, first.after_surface_digest);
    assert_eq!(first.events[0], first.records);
    assert!(first.events[1..].iter().all(|count| *count == 0));
    assert_eq!(first.commits, 0);
    assert_eq!(first.invocations, 1);
    assert_eq!(first.disposals, 1);
}

#[derive(Debug, Eq, PartialEq)]
struct Capture {
    checksum: i128,
    record_sequence_digest: i128,
    event_sequence_digest: i128,
    before_surface_digest: i128,
    after_surface_digest: i128,
    records: usize,
    unchanged_records: usize,
    before_kinds: [usize; 6],
    after_kinds: [usize; 6],
    before_geometry: [usize; 3],
    after_geometry: [usize; 3],
    events: [usize; 5],
    commits: usize,
    invocations: usize,
    disposals: usize,
}

fn capture() -> Capture {
    horizontal_shell_promotion::reset_hooks();
    let input = super::fixture::prepare_o24(KsrArchive::new().bytes());
    let positions = record_positions(&input.objects);
    let before_record_digests = record_digests(&input.objects);
    let before_kinds = kind_totals(&input.objects);
    let before_geometry = geometry_totals(&input.objects);
    let before_surface_digest = surface_sequence_digest(&input.objects);
    let before_pointers = pointers(&input.objects);

    let output = horizontal_shell_promotion::prepare(input).unwrap();
    let after_record_digests = record_digests(&output.objects);
    let after_kinds = kind_totals(&output.objects);
    let after_geometry = geometry_totals(&output.objects);
    let after_surface_digest = surface_sequence_digest(&output.objects);
    assert_eq!(before_pointers, pointers(&output.objects));

    let matched = vec![false; positions.len()];
    let record_sequence_digest = record_sequence_digest(
        &positions,
        &matched,
        &before_record_digests,
        &after_record_digests,
    );
    let unchanged_records = before_record_digests
        .iter()
        .zip(&after_record_digests)
        .filter(|(before, after)| before == after)
        .count();
    let event_sequence = horizontal_shell_promotion::events();
    let event_sequence_digest = event_sequence_digest(&event_sequence);
    let mut events = [0; 5];
    for event in event_sequence {
        events[event as usize] += 1;
    }
    let mut checksum = 0x4f25_4b53_525f_4348_4543_4b53_554d_i128;
    mix(&mut checksum, record_sequence_digest);
    surfaces_digest(&mut checksum, &output.objects);
    mix(&mut checksum, event_sequence_digest);
    let commits = horizontal_shell_promotion::commits();
    let invocations = horizontal_shell_promotion::invocations();
    horizontal_shell_promotion::dispose(output);

    Capture {
        checksum,
        record_sequence_digest,
        event_sequence_digest,
        before_surface_digest,
        after_surface_digest,
        records: positions.len(),
        unchanged_records,
        before_kinds,
        after_kinds,
        before_geometry,
        after_geometry,
        events,
        commits,
        invocations,
        disposals: horizontal_shell_promotion::disposals(),
    }
}

fn surface_sequence_digest(
    objects: &[crate::project_slice::prepare_infill::surface_type_detection::PreparedSurfaceTypeObject],
) -> i128 {
    let mut digest = 0x4f25_5355_5246_4143_455f_4449_4745_5354_i128;
    surfaces_digest(&mut digest, objects);
    digest
}

fn pointers(
    objects: &[crate::project_slice::prepare_infill::surface_type_detection::PreparedSurfaceTypeObject],
) -> Vec<(usize, Vec<usize>)> {
    objects
        .iter()
        .flat_map(|object| &object.records)
        .flatten()
        .map(|record| {
            (
                record.fill_surfaces.as_ptr() as usize,
                record
                    .fill_surfaces
                    .iter()
                    .flat_map(|surface| {
                        let expolygon = surface.as_parts().1;
                        std::iter::once(expolygon.contour()).chain(expolygon.holes())
                    })
                    .map(|path| path.points().as_ptr() as usize)
                    .collect(),
            )
        })
        .collect()
}

fn kind_totals(
    objects: &[crate::project_slice::prepare_infill::surface_type_detection::PreparedSurfaceTypeObject],
) -> [usize; 6] {
    let mut totals = [0; 6];
    for surface in objects
        .iter()
        .flat_map(|object| &object.records)
        .flatten()
        .flat_map(|record| &record.fill_surfaces)
    {
        totals[kind_index(surface.as_parts().0)] += 1;
    }
    totals
}

fn geometry_totals(
    objects: &[crate::project_slice::prepare_infill::surface_type_detection::PreparedSurfaceTypeObject],
) -> [usize; 3] {
    let mut totals = [0; 3];
    for expolygon in objects
        .iter()
        .flat_map(|object| &object.records)
        .flatten()
        .flat_map(|record| &record.fill_surfaces)
        .map(|surface| surface.as_parts().1)
    {
        totals[0] += 1;
        totals[1] += expolygon.holes().len();
        totals[2] += expolygon.contour().points().len()
            + expolygon
                .holes()
                .iter()
                .map(|hole| hole.points().len())
                .sum::<usize>();
    }
    totals
}

fn kind_index(kind: RegionSurfaceKind) -> usize {
    match kind {
        RegionSurfaceKind::Top => 0,
        RegionSurfaceKind::Bottom => 1,
        RegionSurfaceKind::BottomBridge => 2,
        RegionSurfaceKind::Internal => 3,
        RegionSurfaceKind::InternalSolid => 4,
        RegionSurfaceKind::InternalBridge => 6,
        RegionSurfaceKind::InternalVoid => 5,
    }
}
