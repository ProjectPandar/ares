use crate::project_slice::perimeters::{
    classic::{
        gap_extrusion::{
            self, GapFillEntity, PreparedGapExtrusionRecord, PreparedPostClassicGapExtrusion,
            coverage,
        },
        materialize::{ExtrusionPath, ExtrusionRole},
        medial_gap::PreparedMedialGapObject,
    },
    prepare_post_classic_gap_extrusion, prepare_post_classic_medial_gap,
};

use super::super::super::super::support::ksr_project;

#[test]
fn task22o14_ksr_gap_extrusion_structure_is_literal_and_repeatable() {
    let source = prepare_post_classic_medial_gap(ksr_project()).unwrap();
    let predecessor = predecessor_snapshot(&source.objects);
    let moved = gap_extrusion::finish(source).unwrap();
    assert_eq!(successor_snapshot(&moved), predecessor);

    let first_output = prepare_post_classic_gap_extrusion(ksr_project()).unwrap();
    assert!(first_output.objects.iter().any(|object| {
        object.records.iter().flatten().any(|record| {
            record
                .surfaces
                .iter()
                .any(|surface| !surface.gap_fill.entities.is_empty())
        })
    }));
    let first = checksum(&first_output);
    let second = checksum(&prepare_post_classic_gap_extrusion(ksr_project()).unwrap());
    assert_eq!(first, second);
}

const OBJECT_BEGIN: i128 = 0x01_4f424a;
const OBJECT_END: i128 = 0x02_4f424a;
const RECORD_BEGIN: i128 = 0x03_524543;
const RECORD_END: i128 = 0x04_524543;
const SURFACE_BEGIN: i128 = 0x05_535552;
const SURFACE_END: i128 = 0x06_535552;
const MEDIAL_BEGIN: i128 = 0x07_4d4544;
const MEDIAL_END: i128 = 0x08_4d4544;
const POLYLINE_BEGIN: i128 = 0x09_504c59;
const POLYLINE_END: i128 = 0x0a_504c59;
const ENTITY_PATH: i128 = 0x0b_504154;
const ENTITY_LOOP: i128 = 0x0c_4c4f4f;
const PATH_BEGIN: i128 = 0x0d_504154;
const PATH_END: i128 = 0x0e_504154;
const COVERAGE_BEGIN: i128 = 0x0f_434f56;
const COVERAGE_END: i128 = 0x10_434f56;
const REMAINING_BEGIN: i128 = 0x11_52454d;
const REMAINING_END: i128 = 0x12_52454d;
const EXPOLYGON_BEGIN: i128 = 0x13_455850;
const EXPOLYGON_END: i128 = 0x14_455850;
const CONTOUR_BEGIN: i128 = 0x15_434f4e;
const CONTOUR_END: i128 = 0x16_434f4e;
const HOLE_BEGIN: i128 = 0x17_484f4c;
const HOLE_END: i128 = 0x18_484f4c;

fn checksum(prepared: &PreparedPostClassicGapExtrusion) -> i128 {
    let mut checksum = 0_i128;
    mix(&mut checksum, prepared.objects.len() as i128);
    for (object, traversal) in prepared.objects.iter().zip(&prepared.predecessor.objects) {
        mix(&mut checksum, OBJECT_BEGIN);
        mix(&mut checksum, object.records.len() as i128);
        let prelude = &traversal.predecessor.predecessor.predecessor.predecessor;
        for (record, input) in object.records.iter().zip(prelude.object.as_parts().1) {
            mix(&mut checksum, RECORD_BEGIN);
            mix(&mut checksum, i128::from(record.is_some()));
            match (record, input) {
                (Some(record), Some(input)) => {
                    let filter = prelude.object.region_options(input).filter_out_gap_fill.0;
                    mix(&mut checksum, i128::from(filter.to_bits()));
                    mix(
                        &mut checksum,
                        i128::from((filter / prepared.predecessor.scale.factor()).to_bits()),
                    );
                    checksum_record(&mut checksum, record, prepared.predecessor.scale);
                }
                (None, None) => {}
                _ => panic!("O14 KSR record alignment is invariant"),
            }
            mix(&mut checksum, RECORD_END);
        }
        mix(&mut checksum, OBJECT_END);
    }
    checksum
}

fn checksum_record(
    checksum: &mut i128,
    record: &PreparedGapExtrusionRecord,
    scale: crate::geometry::CoordinateScale,
) {
    mix(checksum, record.surfaces.len() as i128);
    for surface in &record.surfaces {
        mix(checksum, SURFACE_BEGIN);
        mix(checksum, surface.source_index as i128);
        match &surface.medial {
            Some(domain) => checksum_medial(checksum, domain),
            None => mix(checksum, 0),
        }
        mix(checksum, surface.gap_fill.entities.len() as i128);
        for entity in &surface.gap_fill.entities {
            checksum_entity(checksum, entity);
        }
        mix(checksum, COVERAGE_BEGIN);
        let covered = coverage::covered_polygons(&surface.gap_fill, scale).unwrap();
        mix(checksum, covered.len() as i128);
        for polygon in covered {
            checksum_points(checksum, polygon.points());
        }
        mix(checksum, COVERAGE_END);
        mix(checksum, REMAINING_BEGIN);
        checksum_expolygons(checksum, &surface.remaining);
        mix(checksum, REMAINING_END);
        mix(checksum, SURFACE_END);
    }
}

fn checksum_medial(
    checksum: &mut i128,
    domain: &crate::project_slice::perimeters::classic::medial_gap::MedialGapDomain,
) {
    mix(checksum, MEDIAL_BEGIN);
    mix(checksum, i128::from(domain.predecessor.min.to_bits()));
    mix(checksum, i128::from(domain.predecessor.max.to_bits()));
    checksum_expolygons(checksum, &domain.predecessor.expolygons);
    mix(checksum, domain.polylines.len() as i128);
    for polyline in &domain.polylines {
        mix(checksum, POLYLINE_BEGIN);
        checksum_points(checksum, &polyline.points);
        mix(checksum, polyline.width.len() as i128);
        for width in &polyline.width {
            mix(checksum, i128::from(width.to_bits()));
        }
        mix(checksum, i128::from(polyline.endpoints.0));
        mix(checksum, i128::from(polyline.endpoints.1));
        mix(checksum, POLYLINE_END);
    }
    mix(checksum, MEDIAL_END);
}

fn checksum_entity(checksum: &mut i128, entity: &GapFillEntity) {
    match entity {
        GapFillEntity::Path(path) => {
            mix(checksum, ENTITY_PATH);
            checksum_path(checksum, path);
        }
        GapFillEntity::Loop(paths) => {
            mix(checksum, ENTITY_LOOP);
            mix(checksum, paths.len() as i128);
            for path in paths {
                checksum_path(checksum, path);
            }
        }
    }
}

fn checksum_path(checksum: &mut i128, path: &ExtrusionPath) {
    mix(checksum, PATH_BEGIN);
    mix(
        checksum,
        match path.role {
            ExtrusionRole::ExternalPerimeter => 1,
            ExtrusionRole::Perimeter => 2,
            ExtrusionRole::OverhangPerimeter => 3,
            ExtrusionRole::GapFill => 4,
            ExtrusionRole::SolidInfill => 5,
            ExtrusionRole::TopSolidInfill => 6,
            ExtrusionRole::BottomSurface => 7,
        },
    );
    mix(checksum, i128::from(path.mm3_per_mm.to_bits()));
    mix(checksum, i128::from(path.width.to_bits()));
    mix(checksum, i128::from(path.height.to_bits()));
    mix(checksum, path.polyline.points.len() as i128);
    for point in &path.polyline.points {
        mix(checksum, i128::from(point.x));
        mix(checksum, i128::from(point.y));
        mix(checksum, i128::from(point.z));
    }
    mix(checksum, PATH_END);
}

fn checksum_expolygons(checksum: &mut i128, expolygons: &[crate::geometry::ExPolygon]) {
    mix(checksum, expolygons.len() as i128);
    for expolygon in expolygons {
        mix(checksum, EXPOLYGON_BEGIN);
        mix(checksum, CONTOUR_BEGIN);
        checksum_points(checksum, expolygon.contour().points());
        mix(checksum, CONTOUR_END);
        mix(checksum, expolygon.holes().len() as i128);
        for hole in expolygon.holes() {
            mix(checksum, HOLE_BEGIN);
            checksum_points(checksum, hole.points());
            mix(checksum, HOLE_END);
        }
        mix(checksum, EXPOLYGON_END);
    }
}

fn checksum_points(checksum: &mut i128, points: &[crate::geometry::Point]) {
    mix(checksum, points.len() as i128);
    for point in points {
        mix(checksum, i128::from(point.x()));
        mix(checksum, i128::from(point.y()));
    }
}

fn predecessor_snapshot(objects: &[PreparedMedialGapObject]) -> Vec<String> {
    objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
        .map(|surface| {
            format!(
                "{:?}|{:?}|{:?}|{:?}",
                surface.source_index, surface.inactive, surface.appended, surface.medial
            )
        })
        .collect()
}

fn successor_snapshot(prepared: &PreparedPostClassicGapExtrusion) -> Vec<String> {
    prepared
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
        .map(|surface| {
            format!(
                "{:?}|{:?}|{:?}|{:?}",
                surface.source_index, surface.inactive, surface.appended, surface.medial
            )
        })
        .collect()
}

fn mix(checksum: &mut i128, value: i128) {
    *checksum = checksum.wrapping_mul(257).wrapping_add(value);
}
