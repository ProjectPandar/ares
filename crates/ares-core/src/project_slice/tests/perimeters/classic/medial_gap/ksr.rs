use crate::project_slice::perimeters::{
    classic::{
        gap_domain::PreparedGapDomainObject,
        medial_gap::{self, PreparedMedialGapObject, PreparedMedialGapRecord},
    },
    prepare_post_classic_gap_domain, prepare_post_classic_medial_gap,
};

use super::super::super::super::support::ksr_project;

#[test]
fn task22o13_ksr_medial_axis_structure_is_literal_and_repeatable() {
    let predecessor = prepare_post_classic_gap_domain(ksr_project()).unwrap();
    let predecessor_snapshot = predecessor_content_gap(&predecessor.objects);
    let moved = medial_gap::finish(predecessor).unwrap();
    assert_eq!(
        predecessor_content_medial(&moved.objects),
        predecessor_snapshot
    );
    let first_output = prepare_post_classic_medial_gap(ksr_project()).unwrap();
    assert!(has_medial_axis(&first_output.objects));
    let first = checksum(&first_output.objects);
    let second = checksum(
        &prepare_post_classic_medial_gap(ksr_project())
            .unwrap()
            .objects,
    );
    assert_eq!(first, second);
    assert_eq!(first, -28_294_579_168_999_590_030_830_581_414_559_852_385);
}

const OBJECT_BEGIN: i128 = 0x01_4f424a;
const OBJECT_END: i128 = 0x02_4f424a;
const RECORD_BEGIN: i128 = 0x03_524543;
const RECORD_END: i128 = 0x04_524543;
const SURFACE_BEGIN: i128 = 0x05_535552;
const SURFACE_END: i128 = 0x06_535552;
const EXPOLYGON_BEGIN: i128 = 0x07_455850;
const EXPOLYGON_END: i128 = 0x08_455850;
const CONTOUR_BEGIN: i128 = 0x09_434f4e;
const CONTOUR_END: i128 = 0x0a_434f4e;
const HOLE_BEGIN: i128 = 0x0b_484f4c;
const HOLE_END: i128 = 0x0c_484f4c;
const POLYLINE_BEGIN: i128 = 0x0d_504c59;
const POLYLINE_END: i128 = 0x0e_504c59;

fn has_medial_axis(objects: &[PreparedMedialGapObject]) -> bool {
    objects.iter().any(|object| {
        object.records.iter().flatten().any(|record| {
            record.surfaces.iter().any(|surface| {
                surface
                    .medial
                    .as_ref()
                    .is_some_and(|domain| !domain.polylines.is_empty())
            })
        })
    })
}

fn checksum(objects: &[PreparedMedialGapObject]) -> i128 {
    let mut checksum = 0_i128;
    mix(&mut checksum, objects.len() as i128);
    for object in objects {
        mix(&mut checksum, OBJECT_BEGIN);
        mix(&mut checksum, object.records.len() as i128);
        for record in &object.records {
            mix(&mut checksum, RECORD_BEGIN);
            mix(&mut checksum, i128::from(record.is_some()));
            if let Some(record) = record {
                checksum_record(&mut checksum, record);
            }
            mix(&mut checksum, RECORD_END);
        }
        mix(&mut checksum, OBJECT_END);
    }
    checksum
}

fn checksum_record(checksum: &mut i128, record: &PreparedMedialGapRecord) {
    mix(checksum, record.surfaces.len() as i128);
    for surface in &record.surfaces {
        mix(checksum, SURFACE_BEGIN);
        mix(checksum, surface.source_index as i128);
        let Some(domain) = &surface.medial else {
            mix(checksum, 0);
            mix(checksum, SURFACE_END);
            continue;
        };
        mix(checksum, 1);
        mix(checksum, i128::from(domain.predecessor.min.to_bits()));
        mix(checksum, i128::from(domain.predecessor.max.to_bits()));
        mix(checksum, domain.predecessor.expolygons.len() as i128);
        for expolygon in &domain.predecessor.expolygons {
            mix(checksum, EXPOLYGON_BEGIN);
            mix(checksum, CONTOUR_BEGIN);
            mix(checksum, expolygon.contour().points().len() as i128);
            checksum_points(checksum, expolygon.contour().points());
            mix(checksum, CONTOUR_END);
            mix(checksum, expolygon.holes().len() as i128);
            for hole in expolygon.holes() {
                mix(checksum, HOLE_BEGIN);
                mix(checksum, hole.points().len() as i128);
                checksum_points(checksum, hole.points());
                mix(checksum, HOLE_END);
            }
            mix(checksum, EXPOLYGON_END);
        }
        mix(checksum, domain.polylines.len() as i128);
        for polyline in &domain.polylines {
            mix(checksum, POLYLINE_BEGIN);
            mix(checksum, polyline.points.len() as i128);
            checksum_points(checksum, &polyline.points);
            mix(checksum, polyline.width.len() as i128);
            for width in &polyline.width {
                mix(checksum, i128::from(width.to_bits()));
            }
            mix(checksum, i128::from(polyline.endpoints.0));
            mix(checksum, i128::from(polyline.endpoints.1));
            mix(checksum, POLYLINE_END);
        }
        mix(checksum, SURFACE_END);
    }
}

fn predecessor_content_gap(objects: &[PreparedGapDomainObject]) -> Vec<String> {
    objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
        .map(|surface| {
            format!(
                "{:?}|{:?}|{:?}",
                surface.inactive, surface.appended, surface.pre_medial
            )
        })
        .collect()
}

fn predecessor_content_medial(objects: &[PreparedMedialGapObject]) -> Vec<String> {
    objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
        .map(|surface| {
            format!(
                "{:?}|{:?}|{:?}",
                surface.inactive,
                surface.appended,
                surface.medial.as_ref().map(|domain| &domain.predecessor)
            )
        })
        .collect()
}

fn checksum_points(checksum: &mut i128, points: &[crate::geometry::Point]) {
    for point in points {
        mix(checksum, i128::from(point.x()));
        mix(checksum, i128::from(point.y()));
    }
}

fn mix(checksum: &mut i128, value: i128) {
    *checksum = checksum.wrapping_mul(257).wrapping_add(value);
}
