use crate::{
    geometry::{CoordinateScale, ExPolygon},
    project_slice::{prepare_infill::vertical_shell_filtering, tests::support::KsrArchive},
};

const O22_CHECKSUM: i128 = 134_936_948_052_282_121_922_360_252_649_864_225_707;
const O22_TOTALS: [usize; 8] = [1, 460, 0, 460, 632, 632, 128, 34_557];
const O22_EVENTS: [usize; 4] = [259, 259, 259, 259];
const O22_RADII_DIGEST: i128 = -119_839_535_044_106_185_061_007_902_266_478_724_784;
const O23_CHECKSUM: i128 = -41_564_956_609_250_807_593_946_297_629_749_369_320;
const O23_TOTALS: [usize; 10] = [1, 460, 0, 460, 632, 554, 78, 554, 128, 33_815];
const O23_THRESHOLD_DIGEST: i128 = -167_664_109_034_474_951_983_490_568_976_349_754_300;
const O23_EVENTS: [usize; 8] = [259, 259, 259, 632, 66, 80, 80, 259];

#[test]
fn task22o23_ksr_filtering_is_parent_guarded_and_repeatable() {
    let first = capture();
    assert_eq!(
        first,
        (O23_CHECKSUM, O23_TOTALS, O23_THRESHOLD_DIGEST, O23_EVENTS)
    );
    let second = capture();
    assert_eq!(
        second,
        (O23_CHECKSUM, O23_TOTALS, O23_THRESHOLD_DIGEST, O23_EVENTS)
    );
}

pub(in crate::project_slice) fn capture() -> (i128, [usize; 10], i128, [usize; 8]) {
    let parent = super::super::vertical_shell_regularization::ksr::capture();
    assert_eq!(
        parent,
        (O22_CHECKSUM, O22_TOTALS, O22_EVENTS, O22_RADII_DIGEST)
    );
    vertical_shell_filtering::reset_geometry_hooks();
    let output = super::fixture::prepare(KsrArchive::new().bytes());
    let actual_parent = super::super::vertical_shell_regularization::ksr::o22_checksum_parts(
        super::super::vertical_shell_regularization::ksr::O22ChecksumParts {
            predecessor: &output.predecessor,
            objects: &output.objects,
            caches: &output.caches,
            projections: &output.projections,
            trims: &output.trims,
            regularizations: &output.regularizations,
        },
    );
    assert_eq!(actual_parent, O22_CHECKSUM);
    let mut checksum = 0x4f32_335f_5041_5245_4e54_i128;
    mix(&mut checksum, actual_parent);
    mix(&mut checksum, filter_digest(&output.filters));
    let captured = (
        checksum,
        totals(&output),
        threshold_digest(&output),
        event_totals(),
    );
    vertical_shell_filtering::dispose(output);
    captured
}

fn filter_digest(
    objects: &[vertical_shell_filtering::types::VerticalShellTinyFilterObject],
) -> i128 {
    let mut digest = 0x4f23_i128;
    mix(&mut digest, objects.len() as i128);
    for object in objects {
        mix(&mut digest, 0x004f_424a);
        mix(&mut digest, object.records.len() as i128);
        for record in &object.records {
            match record {
                None => mix(&mut digest, -1),
                Some(filter) => {
                    mix(&mut digest, 1);
                    expolygons(&mut digest, &filter.filtered_shell);
                }
            }
        }
    }
    digest
}

fn totals(output: &vertical_shell_filtering::PreparedPostVerticalShellFiltering) -> [usize; 10] {
    let mut totals = [output.filters.len(), 0, 0, 0, 0, 0, 0, 0, 0, 0];
    for (filter_object, regularization_object) in output.filters.iter().zip(&output.regularizations)
    {
        totals[1] += filter_object.records.len();
        totals[2] += filter_object
            .records
            .iter()
            .filter(|record| record.is_none())
            .count();
        for (filter, regularization) in filter_object
            .records
            .iter()
            .zip(&regularization_object.records)
        {
            if let (Some(filter), Some(regularization)) = (filter, regularization) {
                add_record_totals(
                    &mut totals,
                    regularization.regularized_shell.len(),
                    &filter.filtered_shell,
                );
            }
        }
    }
    totals
}

fn add_record_totals(totals: &mut [usize; 10], input_len: usize, survivors: &[ExPolygon]) {
    totals[3] += 1;
    totals[4] += input_len;
    totals[5] += survivors.len();
    totals[6] += input_len - survivors.len();
    for expolygon in survivors {
        totals[7] += 1;
        totals[8] += expolygon.holes().len();
        totals[9] += point_count(expolygon);
    }
}

fn threshold_digest(output: &vertical_shell_filtering::PreparedPostVerticalShellFiltering) -> i128 {
    let mut digest = 0x0054_4852_4553_484f_4c44_i128;
    mix(&mut digest, output.predecessor.objects.len() as i128);
    for traversal in &output.predecessor.objects {
        mix(&mut digest, 0x004f_424a);
        let records = &traversal
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .records;
        mix(&mut digest, records.len() as i128);
        for record in records {
            mix_threshold_record(
                &mut digest,
                record.as_ref().map(|record| record.solid_infill_spacing),
                output.predecessor.scale,
            );
        }
    }
    digest
}

fn mix_threshold_record(digest: &mut i128, spacing: Option<i64>, scale: CoordinateScale) {
    let Some(spacing) = spacing else {
        mix(digest, -1);
        return;
    };
    mix(digest, spacing as i128);
    for bits in vertical_shell_filtering::threshold_bits(spacing, scale) {
        mix(digest, bits as i128);
    }
    mix(
        digest,
        vertical_shell_filtering::epsilon_bits(scale) as i128,
    );
}

fn event_totals() -> [usize; 8] {
    let mut totals = [0; 8];
    for event in vertical_shell_filtering::geometry_events() {
        totals[event as usize] += 1;
    }
    totals
}

fn expolygons(digest: &mut i128, expolygons: &[ExPolygon]) {
    mix(digest, expolygons.len() as i128);
    for expolygon in expolygons {
        for path in std::iter::once(expolygon.contour()).chain(expolygon.holes()) {
            mix(digest, path.points().len() as i128);
            for point in path.points() {
                mix(digest, point.x() as i128);
                mix(digest, point.y() as i128);
            }
        }
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

fn mix(checksum: &mut i128, value: i128) {
    *checksum = checksum
        .wrapping_mul(0x1000003d)
        .wrapping_add(value)
        .rotate_left(11);
}
