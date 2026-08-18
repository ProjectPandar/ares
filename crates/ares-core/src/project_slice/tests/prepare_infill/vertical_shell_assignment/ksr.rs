mod digest;

use crate::{
    geometry::ExPolygon,
    project_slice::{
        prepare_infill::{vertical_shell_assignment, vertical_shell_filtering},
        region_slices::RegionSurfaceKind,
        tests::support::KsrArchive,
    },
};
use digest::{
    event_sequence_digest, mix, record_digests, record_positions, record_sequence_digest,
    surfaces_digest,
};

#[test]
fn task22o24_ksr_assignment_is_repeatable_and_has_no_void_producer() {
    assert_ksr_evidence();
}

pub(in crate::project_slice::tests::prepare_infill) fn assert_ksr_evidence() {
    let first = capture();
    let second = capture();
    assert_eq!(second, first);
    assert_eq!(first.void_counts, [0, 0]);
    assert_eq!(
        first.active_records + first.no_op_records,
        first.total_records
    );
    assert_eq!(first.unchanged_no_op_records, first.no_op_records);
}

#[derive(Debug, Eq, PartialEq)]
struct Capture {
    checksum: i128,
    parent_totals: [usize; 10],
    pre_kinds: [usize; 6],
    post_kinds: [usize; 6],
    pre_geometry: [usize; 3],
    post_geometry: [usize; 3],
    total_records: usize,
    active_records: usize,
    no_op_records: usize,
    unchanged_records: usize,
    unchanged_no_op_records: usize,
    record_sequence_digest: i128,
    event_sequence_digest: i128,
    events: [usize; 3],
    void_counts: [usize; 2],
}

fn capture() -> Capture {
    vertical_shell_assignment::reset_geometry_hooks();
    let input = super::fixture::prepare_o23(KsrArchive::new().bytes());
    let parent_totals = filter_totals(&input);
    let pre_kinds = kind_totals(&input.objects);
    let pre_geometry = geometry_totals(&input.objects);
    let active_markers: Vec<_> = input
        .filters
        .iter()
        .flat_map(|object| &object.records)
        .map(|filter| {
            filter
                .as_ref()
                .is_some_and(|filter| !filter.filtered_shell.is_empty())
        })
        .collect();
    let record_positions = record_positions(&input.objects);
    let pre_record_digests = record_digests(&input.objects);
    let output = vertical_shell_assignment::prepare(input).unwrap();
    let post_record_digests = record_digests(&output.objects);
    let total_records = active_markers.len();
    let active_records = active_markers.iter().filter(|&&active| active).count();
    let no_op_records = total_records - active_records;
    let unchanged_records = pre_record_digests
        .iter()
        .zip(&post_record_digests)
        .filter(|(before, after)| before == after)
        .count();
    let unchanged_no_op_records = active_markers
        .iter()
        .zip(&pre_record_digests)
        .zip(&post_record_digests)
        .filter(|((active, before), after)| !**active && before == after)
        .count();
    let record_sequence_digest = record_sequence_digest(
        &record_positions,
        &active_markers,
        &pre_record_digests,
        &post_record_digests,
    );
    let mut checksum = 0x0000_4f32_345f_5041_5245_4e54_i128;
    mix(&mut checksum, record_sequence_digest);
    surfaces_digest(&mut checksum, &output.objects);
    let post_kinds = kind_totals(&output.objects);
    let post_geometry = geometry_totals(&output.objects);
    let event_sequence = vertical_shell_assignment::geometry_events();
    let event_sequence_digest = event_sequence_digest(&event_sequence);
    mix(&mut checksum, event_sequence_digest);
    let mut events = [0; 3];
    for event in event_sequence {
        events[event as usize] += 1;
    }
    vertical_shell_assignment::dispose(output);
    Capture {
        checksum,
        parent_totals,
        pre_kinds,
        post_kinds,
        pre_geometry,
        post_geometry,
        total_records,
        active_records,
        no_op_records,
        unchanged_records,
        unchanged_no_op_records,
        record_sequence_digest,
        event_sequence_digest,
        events,
        void_counts: [pre_kinds[5], post_kinds[5]],
    }
}

fn filter_totals(
    output: &vertical_shell_filtering::PreparedPostVerticalShellFiltering,
) -> [usize; 10] {
    let mut totals = [output.filters.len(), 0, 0, 0, 0, 0, 0, 0, 0, 0];
    for (filters, regularizations) in output.filters.iter().zip(&output.regularizations) {
        totals[1] += filters.records.len();
        totals[2] += filters
            .records
            .iter()
            .filter(|record| record.is_none())
            .count();
        for (filter, regularization) in filters.records.iter().zip(&regularizations.records) {
            if let (Some(filter), Some(regularization)) = (filter, regularization) {
                add_filter_totals(
                    &mut totals,
                    regularization.regularized_shell.len(),
                    &filter.filtered_shell,
                );
            }
        }
    }
    totals
}

fn add_filter_totals(totals: &mut [usize; 10], input_len: usize, output: &[ExPolygon]) {
    totals[3] += 1;
    totals[4] += input_len;
    totals[5] += output.len();
    totals[6] += input_len - output.len();
    for expolygon in output {
        totals[7] += 1;
        totals[8] += expolygon.holes().len();
        totals[9] += point_count(expolygon);
    }
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
        totals[2] += point_count(expolygon);
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

fn point_count(expolygon: &ExPolygon) -> usize {
    expolygon.contour().points().len()
        + expolygon
            .holes()
            .iter()
            .map(|hole| hole.points().len())
            .sum::<usize>()
}
