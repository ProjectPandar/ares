use crate::project_slice::{
    incomplete_sink,
    perimeters::{
        classic::{gap_extrusion, medial_gap::PreparedPostClassicMedialGap},
        prepare_post_classic_medial_gap,
    },
};

use super::super::super::super::support::KsrArchive;

const CONFIG: &str = "Metadata/project_settings.config";

#[test]
fn task22o14_aligned_objects_use_distinct_effective_typed_thresholds_in_order() {
    let baseline = source_with_filter("0");
    let baseline_entities = entity_count(&baseline.objects);
    assert!(baseline_entities > 0);

    let combined = combine(baseline, source_with_filter("1000"));
    let output = gap_extrusion::finish(combined).unwrap();

    assert_eq!(output.objects.len(), 2);
    assert!(entity_count_output(&output.objects[..1]) > 0);
    assert_eq!(entity_count_output(&output.objects[1..]), 0);
    let filtered_domains = output.objects[1]
        .records
        .iter()
        .flatten()
        .flat_map(|record| &record.surfaces)
        .filter_map(|surface| surface.medial.as_ref())
        .collect::<Vec<_>>();
    assert!(!filtered_domains.is_empty());
    assert!(
        filtered_domains
            .iter()
            .all(|domain| domain.polylines.is_empty())
    );
}

pub(super) fn source_with_filter(value: &str) -> PreparedPostClassicMedialGap {
    let mut archive = KsrArchive::new();
    if value != "0" {
        archive.replace_unique(
            CONFIG,
            "\"filter_out_gap_fill\": \"0\"",
            &format!("\"filter_out_gap_fill\": \"{value}\""),
        );
    }
    prepare_post_classic_medial_gap(&archive.bytes()).unwrap()
}

pub(super) fn combine(
    mut first: PreparedPostClassicMedialGap,
    second: PreparedPostClassicMedialGap,
) -> PreparedPostClassicMedialGap {
    let PreparedPostClassicMedialGap {
        mut predecessor,
        mut objects,
    } = second;
    first.objects.append(&mut objects);
    first.predecessor.objects.append(&mut predecessor.objects);
    incomplete_sink::consume_boxed_post_classic_traversal(predecessor);
    first
}

fn entity_count(
    objects: &[crate::project_slice::perimeters::classic::medial_gap::PreparedMedialGapObject],
) -> usize {
    objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
        .filter_map(|surface| surface.medial.as_ref())
        .map(|domain| domain.polylines.len())
        .sum()
}

fn entity_count_output(
    objects: &[crate::project_slice::perimeters::classic::gap_extrusion::PreparedGapExtrusionObject],
) -> usize {
    objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
        .map(|surface| surface.gap_fill.entities.len())
        .sum()
}
