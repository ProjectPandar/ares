use crate::{
    geometry::{ExPolygon, Point},
    project_slice::{
        perimeters::{
            classic::PreparedPostClassicInfillBoundary, prepare_post_classic_infill_boundary,
        },
        region_slices::RegionSurfaceKind,
    },
};

use super::super::super::super::support::ksr_project;

const OBJECT: i128 = 0x01_4f424a;
const RECORD: i128 = 0x02_524543;
const SURFACE: i128 = 0x03_535552;
const INTERNAL: i128 = 0x04_494e54;
const NO_OVERLAP: i128 = 0x05_4e4f4f;
const EXPOLYGON: i128 = 0x06_455850;

#[test]
fn task22o15_ksr_infill_boundary_structure_is_literal_and_repeatable() {
    let first_output = prepare_post_classic_infill_boundary(ksr_project()).unwrap();
    assert!(first_output.objects.iter().any(|object| {
        object
            .records
            .iter()
            .flatten()
            .any(|record| !record.fill_surfaces.is_empty() && !record.fill_no_overlap.is_empty())
    }));
    let first = checksum(&first_output);
    let second = checksum(&prepare_post_classic_infill_boundary(ksr_project()).unwrap());
    assert_eq!(first, second);
    assert_eq!(first, 136_197_013_209_006_370_081_121_271_251_125_478_104,);
}

fn checksum(prepared: &PreparedPostClassicInfillBoundary) -> i128 {
    let mut checksum = 0_i128;
    mix(&mut checksum, prepared.objects.len() as i128);
    mix(&mut checksum, prepared.predecessor.objects.len() as i128);
    for object in &prepared.objects {
        mix(&mut checksum, OBJECT);
        mix(&mut checksum, object.records.len() as i128);
        for record in &object.records {
            mix(&mut checksum, RECORD);
            mix(&mut checksum, i128::from(record.is_some()));
            let Some(record) = record else { continue };
            mix(&mut checksum, record.surfaces.len() as i128);
            for (surface, overlap) in record.surfaces.iter().zip(&record.overlap) {
                mix(&mut checksum, SURFACE);
                mix_bytes(&mut checksum, format!("{surface:?}").as_bytes());
                mix(&mut checksum, overlap.source_index as i128);
                mix(&mut checksum, i128::from(overlap.inset));
                mix(&mut checksum, i128::from(overlap.infill_peri_overlap));
                mix(&mut checksum, i128::from(overlap.top_infill_peri_overlap));
                mix(
                    &mut checksum,
                    i128::from(overlap.min_perimeter_infill_spacing),
                );
                mix(
                    &mut checksum,
                    i128::from(overlap.scaled_resolution.to_bits()),
                );
            }
            mix(&mut checksum, record.fill_surfaces.len() as i128);
            for surface in &record.fill_surfaces {
                mix(&mut checksum, INTERNAL);
                let (kind, expolygon, thickness, layers, angle, extra) = surface.as_parts();
                assert_eq!(kind, RegionSurfaceKind::Internal);
                mix(&mut checksum, kind as i128);
                mix(&mut checksum, i128::from(thickness.to_bits()));
                mix(&mut checksum, i128::from(layers));
                mix(&mut checksum, i128::from(angle.to_bits()));
                mix(&mut checksum, i128::from(extra));
                checksum_expolygon(&mut checksum, expolygon);
            }
            mix(&mut checksum, NO_OVERLAP);
            mix(&mut checksum, record.fill_no_overlap.len() as i128);
            for expolygon in &record.fill_no_overlap {
                checksum_expolygon(&mut checksum, expolygon);
            }
        }
    }
    checksum
}

fn checksum_expolygon(checksum: &mut i128, expolygon: &ExPolygon) {
    mix(checksum, EXPOLYGON);
    checksum_points(checksum, expolygon.contour().points());
    mix(checksum, expolygon.holes().len() as i128);
    for hole in expolygon.holes() {
        checksum_points(checksum, hole.points());
    }
}

fn checksum_points(checksum: &mut i128, points: &[Point]) {
    mix(checksum, points.len() as i128);
    for point in points {
        mix(checksum, i128::from(point.x()));
        mix(checksum, i128::from(point.y()));
    }
}

fn mix_bytes(checksum: &mut i128, bytes: &[u8]) {
    mix(checksum, bytes.len() as i128);
    for &byte in bytes {
        mix(checksum, i128::from(byte));
    }
}

fn mix(checksum: &mut i128, value: i128) {
    *checksum = checksum.wrapping_mul(257).wrapping_add(value);
}
