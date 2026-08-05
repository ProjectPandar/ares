use crate::{
    geometry::ExPolygon,
    project_slice::{
        perimeters::prepare_post_layer_region_perimeters,
        prepare_infill::surface_type_detection::PreparedPostSurfaceTypeDetection,
        region_slices::RegionSurfaceKind, tests::perimeters::layer_region::ksr as o16,
    },
};

use super::{super::super::support::KsrArchive, fixture::prepare};

const O16_CHECKSUM: i128 = -169_716_507_603_417_685_621_692_788_651_154_411_580;
const O16_TOTALS: [usize; 9] = [1, 460, 460, 2_881, 5_243, 2_285, 1_112, 1_112, 1_112];
const EXPECTED_CHECKSUM: i128 = -126_362_407_653_399_901_571_400_348_049_652_748_978;
const EXPECTED_TOTALS: [usize; 24] = [
    1, 460, 460, 2_881, 5_243, 2_285, 1_112, 1_112, 5_388, 519, 6, 666, 4_197, 1_294, 113, 6, 48,
    1_127, 5_388, 517, 85_886, 1_294, 168, 46_011,
];

#[test]
fn task22o17_ksr_surface_types_are_literal_and_repeatable() {
    let predecessor = prepare_post_layer_region_perimeters(KsrArchive::new().bytes()).unwrap();
    assert_eq!(o16::checksum(&predecessor), O16_CHECKSUM);
    assert_eq!(o16::totals(&predecessor), O16_TOTALS);

    let first = prepare(KsrArchive::new().bytes());
    let second = prepare(KsrArchive::new().bytes());
    let first_snapshot = (checksum(&first), totals(&first));
    assert_eq!(first_snapshot, (checksum(&second), totals(&second)));
    assert_eq!(
        first_snapshot,
        (EXPECTED_CHECKSUM, EXPECTED_TOTALS),
        "PARENT_CAPTURE_REQUIRED_O17_CHECKSUM_TOTALS"
    );
}

pub(super) fn checksum(prepared: &PreparedPostSurfaceTypeDetection) -> i128 {
    let mut checksum = O16_CHECKSUM;
    mix(&mut checksum, prepared.objects.len() as i128);
    for (object, traversal) in prepared.objects.iter().zip(&prepared.predecessor.objects) {
        mix(&mut checksum, 0x01_4f424a);
        let input = &traversal
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object;
        let identity = input.identity();
        mix(&mut checksum, identity.0 as i128);
        mix(&mut checksum, identity.1 as i128);
        mix(&mut checksum, object.records.len() as i128);
        for (record, input) in object.records.iter().zip(&input.records) {
            mix(&mut checksum, 0x02_534c54);
            mix(&mut checksum, i128::from(record.is_some()));
            match (record, input) {
                (Some(record), Some(input)) => checksum_record(&mut checksum, record, input),
                (None, None) => {}
                _ => panic!("O17 KSR record alignment is invariant"),
            }
            mix(&mut checksum, 0x03_534c54);
        }
        mix(&mut checksum, 0x04_4f424a);
    }
    checksum
}

fn checksum_record(
    checksum: &mut i128,
    record: &crate::project_slice::prepare_infill::surface_type_detection::types::PreparedSurfaceTypeRecord,
    input: &crate::project_slice::perimeters::types::PerimeterInputRecord,
) {
    for value in [
        input.source_object_index,
        input.transform_index,
        input.planned_layer_index,
        input.layer_id,
        input.region_id,
    ] {
        mix(checksum, value as i128);
    }
    for length in [
        record.perimeters.len(),
        record.thin_fills.len(),
        record.fill_expolygons.len(),
        record.fill_no_overlap_expolygons.len(),
    ] {
        mix(checksum, length as i128);
    }
    checksum_surfaces(checksum, &record.slices);
    checksum_surfaces(checksum, &record.fill_surfaces);
}

fn checksum_surfaces(
    checksum: &mut i128,
    surfaces: &[crate::project_slice::region_slices::RegionSurface],
) {
    mix(checksum, surfaces.len() as i128);
    for surface in surfaces {
        let (kind, expolygon, thickness, layers, angle, extra) = surface.as_parts();
        mix(checksum, kind as i128);
        mix(checksum, i128::from(thickness.to_bits()));
        mix(checksum, i128::from(layers));
        mix(checksum, i128::from(angle.to_bits()));
        mix(checksum, i128::from(extra));
        checksum_expolygon(checksum, expolygon);
    }
}

fn checksum_expolygon(checksum: &mut i128, expolygon: &ExPolygon) {
    checksum_points(checksum, expolygon.contour().points());
    mix(checksum, expolygon.holes().len() as i128);
    for hole in expolygon.holes() {
        checksum_points(checksum, hole.points());
    }
}

fn checksum_points(checksum: &mut i128, points: &[crate::geometry::Point]) {
    mix(checksum, points.len() as i128);
    for point in points {
        mix(checksum, i128::from(point.x()));
        mix(checksum, i128::from(point.y()));
    }
}

pub(super) fn totals(prepared: &PreparedPostSurfaceTypeDetection) -> [usize; 24] {
    let records = prepared
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .collect::<Vec<_>>();
    let mut output = [0; 24];
    output[0] = prepared.objects.len();
    output[1] = prepared
        .objects
        .iter()
        .map(|object| object.records.len())
        .sum();
    output[2] = records.len();
    output[3] = records.iter().map(|record| record.perimeters.len()).sum();
    output[4] = records
        .iter()
        .flat_map(|record| &record.perimeters)
        .map(|collection| collection.entities.len())
        .sum();
    output[5] = records.iter().map(|record| record.thin_fills.len()).sum();
    output[6] = records
        .iter()
        .map(|record| record.fill_expolygons.len())
        .sum();
    output[7] = records
        .iter()
        .map(|record| record.fill_no_overlap_expolygons.len())
        .sum();
    for record in records {
        count_surfaces(&mut output[8..13], &record.slices);
        count_surfaces(&mut output[13..18], &record.fill_surfaces);
        count_geometry(&mut output[18..21], &record.slices);
        count_geometry(&mut output[21..24], &record.fill_surfaces);
    }
    output
}

fn count_surfaces(
    output: &mut [usize],
    surfaces: &[crate::project_slice::region_slices::RegionSurface],
) {
    output[0] += surfaces.len();
    for surface in surfaces {
        output[1 + match surface.as_parts().0 {
            RegionSurfaceKind::Top => 0,
            RegionSurfaceKind::Bottom => 1,
            RegionSurfaceKind::BottomBridge => 2,
            RegionSurfaceKind::Internal => 3,
            RegionSurfaceKind::InternalSolid | RegionSurfaceKind::InternalVoid => {
                panic!("O17 cannot emit internal solid or void surfaces")
            }
        }] += 1;
    }
}

fn count_geometry(
    output: &mut [usize],
    surfaces: &[crate::project_slice::region_slices::RegionSurface],
) {
    for surface in surfaces {
        let expolygon = surface.as_parts().1;
        output[0] += 1;
        output[1] += expolygon.holes().len();
        output[2] += expolygon.contour().points().len()
            + expolygon
                .holes()
                .iter()
                .map(|hole| hole.points().len())
                .sum::<usize>();
    }
}

fn mix(checksum: &mut i128, value: i128) {
    *checksum = checksum.wrapping_mul(257).wrapping_add(value);
}
