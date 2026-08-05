mod digest;

use crate::project_slice::{
    prepare_infill::horizontal_shell_promotion, region_slices::RegionSurfaceKind,
    tests::support::KsrArchive,
};
use digest::{
    event_sequence_digest, mix, record_digests, record_positions, record_sequence_digest,
    surfaces_digest,
};

const O25_CHECKSUM: i128 = 58_727_684_244_877_231_975_278_290_246_623_082_466;
const O25_RECORD_SEQUENCE_DIGEST: i128 = 160_750_122_870_413_723_145_549_886_803_558_415_603;
const O25_EVENT_SEQUENCE_DIGEST: i128 = 95_826_544_899_519_698_779_358_289_371_798_515_623;
const O25_SURFACE_DIGEST: i128 = -107_673_730_348_313_625_723_619_859_456_104_452_971;
const O25_KINDS: [usize; 6] = [113, 6, 48, 1_281, 575, 0];
const O25_GEOMETRY: [usize; 3] = [2_023, 270, 73_848];
const O25_EVENTS: [usize; 5] = [460, 0, 0, 0, 0];

#[test]
fn task22o25_ksr_empty_schedule_is_repeatable_parent_bound_and_allocation_exact() {
    super::super::vertical_shell_assignment::ksr::assert_ksr_evidence();
    let first = capture();
    assert_capture(&first);
    let second = capture();
    assert_capture(&second);
    assert_eq!(first, second);
}

fn assert_capture(capture: &Capture) {
    assert_eq!(
        (
            capture.checksum,
            capture.record_sequence_digest,
            capture.event_sequence_digest,
            capture.before_surface_digest,
        ),
        (
            O25_CHECKSUM,
            O25_RECORD_SEQUENCE_DIGEST,
            O25_EVENT_SEQUENCE_DIGEST,
            O25_SURFACE_DIGEST,
        )
    );
    assert_eq!(capture.records, 460);
    assert_eq!(capture.unchanged_records, 460);
    assert_eq!(capture.before_kinds, O25_KINDS);
    assert_eq!(capture.after_kinds, O25_KINDS);
    assert_eq!(capture.before_geometry, O25_GEOMETRY);
    assert_eq!(capture.after_geometry, O25_GEOMETRY);
    assert_eq!(capture.events, O25_EVENTS);
    assert_eq!(capture.commits, 0);
    assert_eq!(capture.invocations, 1);
    assert_eq!(capture.disposals, 1);
    assert_eq!(capture.before_surface_digest, capture.after_surface_digest);
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
        RegionSurfaceKind::InternalVoid => 5,
    }
}
