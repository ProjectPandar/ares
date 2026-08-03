use crate::project_slice::{
    incomplete_sink,
    perimeters::{
        classic::{
            gap_extrusion::{self, GapFillEntity, PreparedGapExtrusionSurface, coverage},
            medial_gap::PreparedPostClassicMedialGap,
        },
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

#[test]
fn task22o14_aligned_stage_gap_coverage_and_remaining_are_literal() {
    let output = gap_extrusion::finish(source_with_filter("0")).unwrap();
    let surface = output
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
        .find(|surface| !surface.gap_fill.entities.is_empty())
        .unwrap();
    assert_eq!(
        surface_checksum(surface, output.predecessor.scale),
        67_440_474_419_400_664_307_415_468_191_218_722_519,
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
    prepare_post_classic_medial_gap(archive.bytes()).unwrap()
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

fn surface_checksum(
    surface: &PreparedGapExtrusionSurface,
    scale: crate::geometry::CoordinateScale,
) -> i128 {
    let mut checksum = 0_i128;
    mix(&mut checksum, surface.gap_fill.entities.len() as i128);
    for entity in &surface.gap_fill.entities {
        match entity {
            GapFillEntity::Path(path) => {
                mix(&mut checksum, 1);
                checksum_path(&mut checksum, path);
            }
            GapFillEntity::Loop(paths) => {
                mix(&mut checksum, 2);
                mix(&mut checksum, paths.len() as i128);
                for path in paths {
                    checksum_path(&mut checksum, path);
                }
            }
        }
    }
    let covered = coverage::covered_polygons(&surface.gap_fill, scale).unwrap();
    mix(&mut checksum, covered.len() as i128);
    for polygon in covered {
        checksum_points(&mut checksum, polygon.points());
    }
    mix(&mut checksum, surface.remaining.len() as i128);
    for expolygon in &surface.remaining {
        checksum_points(&mut checksum, expolygon.contour().points());
        mix(&mut checksum, expolygon.holes().len() as i128);
        for hole in expolygon.holes() {
            checksum_points(&mut checksum, hole.points());
        }
    }
    checksum
}

fn checksum_path(
    checksum: &mut i128,
    path: &crate::project_slice::perimeters::classic::materialize::ExtrusionPath,
) {
    mix(checksum, i128::from(path.mm3_per_mm.to_bits()));
    mix(checksum, i128::from(path.width.to_bits()));
    mix(checksum, i128::from(path.height.to_bits()));
    mix(checksum, path.polyline.points.len() as i128);
    for point in &path.polyline.points {
        mix(checksum, i128::from(point.x));
        mix(checksum, i128::from(point.y));
        mix(checksum, i128::from(point.z));
    }
}

fn checksum_points(checksum: &mut i128, points: &[crate::geometry::Point]) {
    mix(checksum, points.len() as i128);
    for point in points {
        mix(checksum, i128::from(point.x()));
        mix(checksum, i128::from(point.y()));
    }
}

fn mix(checksum: &mut i128, value: i128) {
    *checksum = checksum.wrapping_mul(257).wrapping_add(value);
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
