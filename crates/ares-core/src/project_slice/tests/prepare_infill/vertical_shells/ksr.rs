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

const O18_CHECKSUM: i128 = -126_362_407_653_399_901_571_400_348_049_652_748_978;
const O18_TOTALS: [usize; 26] = [
    1, 460, 460, 2_881, 5_243, 2_285, 1_112, 1_112, 5_388, 519, 6, 666, 4_197, 1_294, 113, 6, 48,
    1_127, 5_388, 517, 85_886, 1_294, 168, 46_011, 0, 0,
];
const O19_CACHE_CHECKSUM: i128 = -114_359_197_324_258_778_780_701_398_534_712_718_623;
const PARENT_CAPTURE_MARKER: i128 = 0x4f31_395f_5041_5245_4e54;
const O19_SUCCESSOR_CHECKSUM: i128 = 148_296_943_860_974_241_781_127_169_756_103_364_063;
const O19_TOTALS: [usize; 9] = [1, 460, 0, 460, 572, 713, 1_227, 60_370, 2_512];
const O19_SPACINGS: [i64; 2] = [457_079, 377_079];

#[test]
fn task22o19_ksr_cache_is_full_structure_repeatable() {
    vertical_shells::reset_geometry_hooks();
    let first = fixture::prepare(KsrArchive::new().bytes());
    assert_eq!(
        o18_checksum(&first.predecessor, &first.objects),
        O18_CHECKSUM
    );
    assert_eq!(o18_totals(&first.objects), O18_TOTALS);
    let first_digest = cache_digest(&first.caches);
    assert_eq!(first_digest, O19_CACHE_CHECKSUM);
    assert_eq!(successor_checksum(&first), O19_SUCCESSOR_CHECKSUM);
    assert_eq!(cache_totals(&first.caches), O19_TOTALS);
    let prelude = &first.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    assert_eq!(
        [
            prelude.records[0].as_ref().unwrap().solid_infill_spacing,
            prelude.records[1].as_ref().unwrap().solid_infill_spacing,
        ],
        O19_SPACINGS
    );
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
        O18_CHECKSUM
    );
    assert_eq!(o18_totals(&second.objects), O18_TOTALS);
    assert_eq!(cache_digest(&second.caches), O19_CACHE_CHECKSUM);
    assert_eq!(successor_checksum(&second), O19_SUCCESSOR_CHECKSUM);
    assert_eq!(cache_totals(&second.caches), O19_TOTALS);
    assert_eq!(first.caches.len(), 1);
    assert_eq!(first.caches[0].records.len(), 460);
    assert_eq!(first.caches[0].records.iter().flatten().count(), 460);
    assert_ne!(first_digest, 0);
    vertical_shells::reset_geometry_hooks();
}

fn successor_checksum(
    prepared: &crate::project_slice::prepare_infill::vertical_shells::PreparedPostVerticalShellCache,
) -> i128 {
    assert_eq!(prepared.objects.len(), prepared.caches.len());
    for (object, cache) in prepared.objects.iter().zip(&prepared.caches) {
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
    mix(
        &mut checksum,
        o18_checksum(&prepared.predecessor, &prepared.objects),
    );
    mix(&mut checksum, cache_digest(&prepared.caches));
    checksum
}

pub(super) fn cache_totals(objects: &[VerticalShellCacheObject]) -> [usize; 9] {
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

pub(super) fn cache_digest(objects: &[VerticalShellCacheObject]) -> i128 {
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
