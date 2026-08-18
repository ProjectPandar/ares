use crate::{
    geometry::Polygon,
    project_slice::{
        prepare_infill::vertical_shells::{self, GeometryStep, types::VerticalShellCacheObject},
        tests::{
            prepare_infill::fill_surfaces::ksr::{
                checksum::checksum as o18_checksum, totals::totals as o18_totals,
            },
            support::KsrArchive,
        },
    },
};

use super::fixture;

const PARENT_CAPTURE_MARKER: i128 = 0x4f31_395f_5041_5245_4e54;

#[test]
fn task22o19_ksr_cache_is_full_structure_repeatable() {
    vertical_shells::reset_geometry_hooks();
    let first = fixture::prepare(KsrArchive::new().bytes());
    let first_parent_digest = o18_checksum(&first.predecessor, &first.objects);
    let first_parent_totals = o18_totals(&first.objects);
    let first_digest = cache_digest(&first.caches);
    let first_successor_digest = successor_checksum(&first);
    let first_totals = cache_totals(&first.caches);
    assert_ne!(first_parent_digest, 0);
    assert_ne!(first_digest, 0);
    assert_ne!(first_successor_digest, 0);
    let prelude = &first.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let first_spacings = [
        prelude.records[0].as_ref().unwrap().solid_infill_spacing,
        prelude.records[1].as_ref().unwrap().solid_infill_spacing,
    ];
    assert!(first_spacings.into_iter().all(|spacing| spacing > 0));
    let events = vertical_shells::geometry_events();
    assert_eq!(events.len(), 920);
    assert!(
        events
            .chunks_exact(2)
            .all(|events| { events == [GeometryStep::Top, GeometryStep::Bottom] })
    );
    vertical_shells::reset_geometry_hooks();
    let second = fixture::prepare(KsrArchive::new().bytes());
    assert_eq!(
        o18_checksum(&second.predecessor, &second.objects),
        first_parent_digest
    );
    assert_eq!(o18_totals(&second.objects), first_parent_totals);
    assert_eq!(cache_digest(&second.caches), first_digest);
    assert_eq!(successor_checksum(&second), first_successor_digest);
    assert_eq!(cache_totals(&second.caches), first_totals);
    assert_eq!(first.caches.len(), 1);
    assert_eq!(first.caches[0].records.len(), 460);
    assert_eq!(first.caches[0].records.iter().flatten().count(), 460);
    assert_ne!(first_digest, 0);
    vertical_shells::reset_geometry_hooks();
}

pub(in crate::project_slice::tests::prepare_infill) fn successor_checksum(
    prepared: &crate::project_slice::prepare_infill::vertical_shells::PreparedPostVerticalShellCache,
) -> i128 {
    successor_checksum_parts(&prepared.predecessor, &prepared.objects, &prepared.caches)
}

pub(in crate::project_slice::tests::prepare_infill) fn successor_checksum_parts(
    predecessor: &crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
    objects: &[crate::project_slice::prepare_infill::surface_type_detection::PreparedSurfaceTypeObject],
    caches: &[VerticalShellCacheObject],
) -> i128 {
    assert_eq!(objects.len(), caches.len());
    for (object, cache) in objects.iter().zip(caches) {
        assert_eq!(object.records.len(), cache.records.len());
        assert!(
            object
                .records
                .iter()
                .zip(&cache.records)
                .all(|(record, cache)| record.is_some() == cache.is_some())
        );
    }
    let mut checksum = PARENT_CAPTURE_MARKER;
    mix(&mut checksum, o18_checksum(predecessor, objects));
    mix(&mut checksum, cache_digest(caches));
    checksum
}

pub(in crate::project_slice::tests::prepare_infill) fn cache_totals(
    objects: &[VerticalShellCacheObject],
) -> [usize; 9] {
    let mut totals = [objects.len(), 0, 0, 0, 0, 0, 0, 0, 0];
    for object in objects {
        totals[1] += object.records.len();
        totals[2] += object
            .records
            .iter()
            .filter(|record| record.is_none())
            .count();
        for cache in object.records.iter().flatten() {
            totals[3] += 1;
            totals[4] += cache.top_surfaces.len();
            totals[5] += cache.bottom_surfaces.len();
            totals[6] += cache.holes.len();
            totals[7] += cache
                .top_surfaces
                .iter()
                .chain(&cache.bottom_surfaces)
                .chain(&cache.holes)
                .map(|path| path.points().len())
                .sum::<usize>();
            totals[8] += cache.top_surfaces.len() + cache.bottom_surfaces.len() + cache.holes.len();
        }
    }
    totals
}

pub(in crate::project_slice::tests::prepare_infill) fn cache_digest(
    objects: &[VerticalShellCacheObject],
) -> i128 {
    let mut digest = 0x4f19_i128;
    mix(&mut digest, objects.len() as i128);
    for object in objects {
        mix(&mut digest, 0x4f424a);
        mix(&mut digest, object.records.len() as i128);
        for record in &object.records {
            match record {
                None => mix(&mut digest, -1),
                Some(cache) => {
                    mix(&mut digest, 1);
                    paths(&mut digest, &cache.top_surfaces);
                    paths(&mut digest, &cache.bottom_surfaces);
                    paths(&mut digest, &cache.holes);
                }
            }
        }
    }
    digest
}

fn paths(digest: &mut i128, paths: &[Polygon]) {
    mix(digest, paths.len() as i128);
    for path in paths {
        mix(digest, path.points().len() as i128);
        for point in path.points() {
            mix(digest, point.x() as i128);
            mix(digest, point.y() as i128);
        }
    }
}

fn mix(digest: &mut i128, value: i128) {
    *digest = digest.wrapping_mul(0x100_0000_01b3).wrapping_add(value);
}
