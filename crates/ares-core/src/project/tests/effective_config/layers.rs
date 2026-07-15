use crate::{
    Project, load_project,
    project::effective_config::layers::{
        LayerCandidateRange, layer_candidate_ranges, layer_range_source_index,
    },
};

use super::ProjectParts;

const EPSILON: f64 = 1e-4;

#[test]
fn empty_ranges_produce_one_unconfigured_infinite_interval() {
    let project = project_with_ranges(&[]);

    assert_eq!(
        views(layer_candidate_ranges(raw_ranges(&project))),
        [(0.0, f64::MAX, None)]
    );
}

#[test]
fn negative_start_is_clamped_and_keeps_its_source_index() {
    let project = project_with_ranges(&[(-1.0, 0.3)]);

    assert_eq!(
        views(layer_candidate_ranges(raw_ranges(&project))),
        [(0.0, 0.3, Some(0)), (0.3, f64::MAX, None)]
    );
}

#[test]
fn first_lexicographic_ranges_trim_overlaps_and_raw_ranges_stay_unchanged() {
    let bounds = [
        (-1.0, -2.0),
        (0.0, 0.5),
        (0.1, 0.4),
        (0.1, 0.8),
        (0.5, 0.7),
        (1.0, 0.9),
    ];
    let project = project_with_ranges(&bounds);
    let raw = raw_ranges(&project);
    let before = raw_bounds(raw);

    let ranges = layer_candidate_ranges(raw);

    assert_eq!(before, bounds);
    assert_eq!(raw_bounds(raw), before);
    assert_eq!(
        views(ranges),
        [
            (0.0, 0.5, Some(1)),
            (0.5, 0.8, Some(3)),
            (0.8, f64::MAX, None),
        ]
    );
}

#[test]
fn exact_and_tiny_gaps_join_but_a_gap_beyond_epsilon_is_retained() {
    let project = project_with_ranges(&[(0.0, 0.5), (0.5, 0.6), (0.60005, 0.7), (0.7002, 0.8)]);

    assert_eq!(
        views(layer_candidate_ranges(raw_ranges(&project))),
        [
            (0.0, 0.5, Some(0)),
            (0.5, 0.6, Some(1)),
            (0.6, 0.7, Some(2)),
            (0.7, 0.7002, None),
            (0.7002, 0.8, Some(3)),
            (0.8, f64::MAX, None),
        ]
    );

    let boundary_project = project_with_ranges(&[(EPSILON, 0.2)]);
    assert_eq!(
        views(layer_candidate_ranges(raw_ranges(&boundary_project))),
        [(0.0, 0.2, Some(0)), (0.2, f64::MAX, None)]
    );
}

#[test]
fn configured_ranges_at_or_below_epsilon_are_skipped() {
    for end in [EPSILON / 2.0, EPSILON] {
        let project = project_with_ranges(&[(0.0, end)]);

        assert_eq!(
            views(layer_candidate_ranges(raw_ranges(&project))),
            [(0.0, f64::MAX, None)],
            "end {end}"
        );
    }
}

#[test]
fn tiny_configured_range_after_gap_extends_the_existing_unconfigured_tail() {
    let project = project_with_ranges(&[(0.2, 0.20005)]);

    assert_eq!(
        views(layer_candidate_ranges(raw_ranges(&project))),
        [(0.0, f64::MAX, None)]
    );
}

#[test]
fn positive_reversed_range_can_create_a_gap_before_a_later_valid_range() {
    let project = project_with_ranges(&[(1.0, 0.9), (1.2, 1.5)]);

    assert_eq!(
        views(layer_candidate_ranges(raw_ranges(&project))),
        [
            (0.0, 1.0, None),
            (1.0, 1.2, None),
            (1.2, 1.5, Some(1)),
            (1.5, f64::MAX, None),
        ]
    );
}

#[test]
fn range_ending_at_last_z_is_skipped_before_it_can_split_a_later_gap() {
    let project = project_with_ranges(&[(0.0, 0.5), (1.0, 0.5), (1.2, 1.5)]);

    assert_eq!(
        views(layer_candidate_ranges(raw_ranges(&project))),
        [
            (0.0, 0.5, Some(0)),
            (0.5, 1.2, None),
            (1.2, 1.5, Some(2)),
            (1.5, f64::MAX, None),
        ]
    );
}

#[test]
fn lookup_returns_configured_source_identity_and_unconfigured_matches() {
    let project = project_with_ranges(&[(-1.0, -2.0), (0.2, 0.5), (0.6, 0.8)]);
    let ranges = layer_candidate_ranges(raw_ranges(&project));

    assert_eq!(layer_range_source_index(&ranges, (0.2, 0.5)), Some(Some(1)));
    assert_eq!(layer_range_source_index(&ranges, (0.0, 0.2)), Some(None));
    assert_eq!(layer_range_source_index(&ranges, (0.5, 0.6)), Some(None));
    assert_eq!(
        layer_range_source_index(&ranges, (0.8, f64::MAX)),
        Some(None)
    );
}

#[test]
fn lookup_rejects_no_candidate_and_each_bound_mismatch_beyond_epsilon() {
    let project = project_with_ranges(&[(0.2, 0.5)]);
    let ranges = layer_candidate_ranges(raw_ranges(&project));

    assert_eq!(layer_range_source_index(&ranges, (0.6, 0.8)), None);
    assert_eq!(layer_range_source_index(&ranges, (0.1998, 0.5)), None);
    assert_eq!(layer_range_source_index(&ranges, (0.2, 0.4998)), None);
}

#[test]
fn lookup_subtracts_epsilon_and_accepts_within_and_at_the_boundary() {
    let project = project_with_ranges(&[(0.2, 0.5)]);
    let ranges = layer_candidate_ranges(raw_ranges(&project));

    assert_eq!(
        layer_range_source_index(&ranges, (0.2 + EPSILON / 2.0, 0.5 + EPSILON / 2.0)),
        Some(Some(0))
    );

    let boundary_project = project_with_ranges(&[(0.0, EPSILON * 2.0)]);
    let boundary_ranges = layer_candidate_ranges(raw_ranges(&boundary_project));
    assert_eq!(
        layer_range_source_index(&boundary_ranges, (-EPSILON, EPSILON)),
        Some(Some(0))
    );
    assert_eq!(
        layer_range_source_index(&boundary_ranges, (EPSILON, EPSILON * 2.0 + EPSILON / 2.0),),
        Some(Some(0))
    );
}

#[test]
fn lookup_keeps_the_candidate_when_the_shifted_key_equals_its_bounds() {
    let project = project_with_ranges(&[(0.0, 1.0)]);
    let ranges = layer_candidate_ranges(raw_ranges(&project));
    let candidate = ranges[0];
    let requested = (candidate.min_z + EPSILON, candidate.max_z + EPSILON);

    assert_eq!(candidate.source_range_index, Some(0));
    assert_eq!(
        (requested.0 - EPSILON, requested.1 - EPSILON),
        (candidate.min_z, candidate.max_z)
    );
    assert_eq!(layer_range_source_index(&ranges, requested), Some(Some(0)));
}

fn project_with_ranges(bounds: &[(f64, f64)]) -> Project {
    let ranges = bounds
        .iter()
        .enumerate()
        .map(|(index, (min_z, max_z))| {
            format!(
                r#"<range min_z="{min_z}" max_z="{max_z}"><option opt_key="wall_loops">{}</option></range>"#,
                index + 1
            )
        })
        .collect::<String>();
    let xml = format!(r#"<objects><object id="1">{ranges}</object></objects>"#);
    let mut parts = ProjectParts::valid();
    parts.insert_text("Metadata/layer_config_ranges.xml", &xml);
    load_project(parts.bytes()).unwrap()
}

fn raw_ranges(project: &Project) -> &[crate::LayerConfigRange] {
    project.objects()[0].layer_config_ranges()
}

fn raw_bounds(ranges: &[crate::LayerConfigRange]) -> Vec<(f64, f64)> {
    ranges
        .iter()
        .map(|range| (range.min_z(), range.max_z()))
        .collect()
}

fn views(ranges: Vec<LayerCandidateRange>) -> Vec<(f64, f64, Option<usize>)> {
    ranges
        .into_iter()
        .map(|range| (range.min_z, range.max_z, range.source_range_index))
        .collect()
}
