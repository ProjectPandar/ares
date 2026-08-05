use crate::{
    geometry::Polygon,
    project_slice::{
        prepare_infill::{
            horizontal_shell_promotion::PromotionEvent,
            surface_type_detection::PreparedSurfaceTypeObject,
        },
        region_slices::RegionSurface,
    },
};

pub(super) fn record_positions(objects: &[PreparedSurfaceTypeObject]) -> Vec<(usize, usize)> {
    objects
        .iter()
        .enumerate()
        .flat_map(|(object_index, object)| {
            (0..object.records.len()).map(move |slot_index| (object_index, slot_index))
        })
        .collect()
}

pub(super) fn record_digests(objects: &[PreparedSurfaceTypeObject]) -> Vec<i128> {
    objects
        .iter()
        .flat_map(|object| &object.records)
        .map(|record| {
            let mut digest = 0x004f_2552_4543_4f52_445f_4449_4745_5354_i128;
            let Some(record) = record else {
                mix(&mut digest, -1);
                return digest;
            };
            mix(&mut digest, -2);
            mix(&mut digest, record.fill_surfaces.len() as i128);
            for (surface_index, surface) in record.fill_surfaces.iter().enumerate() {
                mix(&mut digest, surface_index as i128);
                surface_digest(&mut digest, surface);
            }
            mix(&mut digest, -3);
            digest
        })
        .collect()
}

pub(super) fn record_sequence_digest(
    positions: &[(usize, usize)],
    matched: &[bool],
    before: &[i128],
    after: &[i128],
) -> i128 {
    assert_eq!(positions.len(), matched.len());
    assert_eq!(matched.len(), before.len());
    assert_eq!(matched.len(), after.len());
    let mut digest = 0x4f25_5245_434f_5244_5f53_4551_5545_4e43_i128;
    mix(&mut digest, matched.len() as i128);
    for (index, (((&(object, slot), &matched), &before), &after)) in positions
        .iter()
        .zip(matched)
        .zip(before)
        .zip(after)
        .enumerate()
    {
        mix(&mut digest, -10);
        mix(&mut digest, index as i128);
        mix(&mut digest, object as i128);
        mix(&mut digest, slot as i128);
        mix(
            &mut digest,
            if matched {
                0x004d_4154_4348_4544
            } else {
                0x4e4f_5f4d_4154_4348
            },
        );
        mix(&mut digest, before);
        mix(&mut digest, after);
        mix(&mut digest, -11);
    }
    digest
}

pub(super) fn event_sequence_digest(events: &[PromotionEvent]) -> i128 {
    let mut digest = 0x4f25_4556_454e_545f_5345_5155_454e_4345_i128;
    mix(&mut digest, events.len() as i128);
    for (index, event) in events.iter().enumerate() {
        mix(&mut digest, -20);
        mix(&mut digest, index as i128);
        mix(&mut digest, *event as i128);
        mix(&mut digest, -21);
    }
    digest
}

pub(super) fn surfaces_digest(digest: &mut i128, objects: &[PreparedSurfaceTypeObject]) {
    mix(digest, objects.len() as i128);
    for (object_index, object) in objects.iter().enumerate() {
        mix(digest, -30);
        mix(digest, object_index as i128);
        mix(digest, object.records.len() as i128);
        for (slot_index, record) in object.records.iter().enumerate() {
            mix(digest, -31);
            mix(digest, slot_index as i128);
            let Some(record) = record else {
                mix(digest, -32);
                continue;
            };
            mix(digest, record.fill_surfaces.len() as i128);
            for (surface_index, surface) in record.fill_surfaces.iter().enumerate() {
                mix(digest, surface_index as i128);
                surface_digest(digest, surface);
            }
            mix(digest, -33);
        }
        mix(digest, -34);
    }
}

fn surface_digest(digest: &mut i128, surface: &RegionSurface) {
    let (kind, expolygon, thickness, layers, angle, extra) = surface.as_parts();
    mix(digest, -40);
    mix(digest, kind as i128);
    mix(digest, thickness.to_bits() as i128);
    mix(digest, layers as i128);
    mix(digest, angle.to_bits() as i128);
    mix(digest, extra as i128);
    mix(digest, (1 + expolygon.holes().len()) as i128);
    path_digest(digest, -41, 0, expolygon.contour());
    for (hole_index, hole) in expolygon.holes().iter().enumerate() {
        path_digest(digest, -42, hole_index, hole);
    }
    mix(digest, -43);
}

fn path_digest(digest: &mut i128, role: i128, index: usize, path: &Polygon) {
    mix(digest, role);
    mix(digest, index as i128);
    mix(digest, path.points().len() as i128);
    for point in path.points() {
        mix(digest, point.x() as i128);
        mix(digest, point.y() as i128);
    }
    mix(digest, -44);
}

pub(super) fn mix(checksum: &mut i128, value: i128) {
    *checksum = checksum
        .wrapping_mul(0x1000003d)
        .wrapping_add(value)
        .rotate_left(11);
}
