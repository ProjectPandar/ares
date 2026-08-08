use super::{EXPAND_EXPOLYGONS, helpers::*};
use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, Polygon, RegionExpansionEx,
    RegionExpansionParameters, WaveSeed, propagate_waves_ex, wave_seeds,
};

const OUTSIDE: i64 = 0x4000_0000_0000_0000;
type SeedSnapshot = (u32, u32, Vec<(i64, i64)>);
type ExpansionSnapshot = (u32, u32, Vec<(i64, i64)>, Vec<Vec<(i64, i64)>>);

#[rustfmt::skip]
const SMALL_ANCHOR: &[(i64, i64)] = &[(30, 30), (20, 30), (20, 20), (30, 20), (30, 30)];
#[rustfmt::skip]
const LARGE_ANCHOR: &[(i64, i64)] = &[(230, 230), (220, 230), (220, 220), (230, 220), (230, 230)];
#[rustfmt::skip]
const SMALL_OUTER: &[(i64, i64)] = &[(32, 20), (32, 30), (30, 32), (20, 32), (18, 30), (18, 20), (20, 18), (30, 18)];
#[rustfmt::skip]
const SMALL_INNER: &[(i64, i64)] = &[(22, 22), (22, 28), (28, 28), (28, 22)];
#[rustfmt::skip]
const LARGE_OUTER: &[(i64, i64)] = &[(232, 220), (232, 230), (230, 232), (220, 232), (218, 230), (218, 220), (220, 218), (230, 218)];
#[rustfmt::skip]
const LARGE_INNER: &[(i64, i64)] = &[(222, 222), (222, 228), (228, 228), (228, 222)];

fn expansion_params() -> RegionExpansionParameters {
    RegionExpansionParameters::build(2.0, 2.0, 1, CoordinateScale::Normal)
}

fn expansion_zone(expolygons: Vec<ExPolygon>) -> super::super::ExpansionZone {
    super::super::ExpansionZone::new(expolygons, expansion_params())
}

fn points(polygon: &Polygon) -> Vec<(i64, i64)> {
    polygon
        .points()
        .iter()
        .map(|point| (point.x(), point.y()))
        .collect()
}

fn seed_snapshots(seeds: &[WaveSeed]) -> Vec<SeedSnapshot> {
    seeds
        .iter()
        .map(|seed| (seed.src, seed.boundary, points(&seed.path)))
        .collect()
}

fn expansion_snapshots(expansions: &[RegionExpansionEx]) -> Vec<ExpansionSnapshot> {
    expansions
        .iter()
        .map(|expansion| {
            (
                expansion.src_id,
                expansion.boundary_id,
                points(expansion.expolygon.contour()),
                expansion.expolygon.holes().iter().map(points).collect(),
            )
        })
        .collect()
}

fn explicit_pipeline(
    sources: &[ExPolygon],
    zones: &mut [super::super::ExpansionZone],
    scale: CoordinateScale,
) -> Result<(Vec<WaveSeed>, Vec<RegionExpansionEx>), ClipperError> {
    let mut anchors = Vec::new();
    let mut expansions = Vec::new();
    let mut processed = 0_u32;
    for zone in zones {
        let mut zone_anchors = wave_seeds(
            sources,
            &zone.expolygons,
            zone.parameters.tiny_expansion,
            true,
            scale,
        )?;
        let mut zone_expansions =
            propagate_waves_ex(&zone_anchors, &zone.expolygons, &zone.parameters)?;
        for anchor in &mut zone_anchors {
            anchor.boundary = anchor.boundary.wrapping_add(processed);
        }
        for expansion in &mut zone_expansions {
            expansion.boundary_id = expansion.boundary_id.wrapping_add(processed);
        }
        zone.expanded_into = !zone_expansions.is_empty();
        anchors.append(&mut zone_anchors);
        expansions.append(&mut zone_expansions);
        processed = processed.wrapping_add(zone.expolygons.len() as u32);
    }
    Ok((anchors, expansions))
}

fn assert_o30_error(sources: &[ExPolygon], zone: &super::super::ExpansionZone) {
    let seeds = wave_seeds(
        sources,
        &zone.expolygons,
        zone.parameters.tiny_expansion,
        true,
        CoordinateScale::Normal,
    )
    .unwrap();
    assert!(!seeds.is_empty());
    assert_eq!(
        propagate_waves_ex(&seeds, &zone.expolygons, &zone.parameters),
        Err(ClipperError::CoordinateOutOfRange)
    );
}

#[test]
fn task22o36_zero_zones_do_not_access_source_and_empty_source_visits_zones() {
    let invalid_source = [expolygon(
        &[(OUTSIDE, 0), (OUTSIDE, 10), (OUTSIDE - 1, 10)],
        Vec::new(),
    )];
    let result = EXPAND_EXPOLYGONS(&invalid_source, &mut [], CoordinateScale::Normal).unwrap();
    assert!(result.anchors.is_empty());
    assert!(result.expansions.is_empty());

    let mut zones = vec![expansion_zone(vec![]), expansion_zone(vec![square(0, 100)])];
    for zone in &mut zones {
        zone.expanded_into = true;
    }
    let result = EXPAND_EXPOLYGONS(&[], &mut zones, CoordinateScale::Normal).unwrap();
    assert!(result.anchors.is_empty());
    assert!(result.expansions.is_empty());
    assert!(zones.iter().all(|zone| !zone.expanded_into));

    zones[1].parameters.tiny_expansion = 0.0;
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            EXPAND_EXPOLYGONS(&[], &mut zones, CoordinateScale::Normal)
        }))
        .is_err()
    );
}

#[test]
fn task22o36_one_natural_zone_preserves_complete_anchor_and_expansion() {
    let sources = [square(20, 30)];
    let mut zones = vec![expansion_zone(vec![square(0, 100)])];
    let result = EXPAND_EXPOLYGONS(&sources, &mut zones, CoordinateScale::Normal).unwrap();

    assert_eq!(
        seed_snapshots(&result.anchors),
        vec![(0, 0, SMALL_ANCHOR.to_vec())]
    );
    assert_eq!(
        expansion_snapshots(&result.expansions),
        vec![(0, 0, SMALL_OUTER.to_vec(), vec![SMALL_INNER.to_vec()])]
    );
    assert!(zones[0].expanded_into);
}

#[test]
fn task22o36_ordered_zones_rebase_complete_pinned_oracle() {
    let sources = [square(220, 230), square(20, 30)];
    let mut zones = vec![
        expansion_zone(vec![square(400, 500)]),
        expansion_zone(vec![square(0, 100)]),
        expansion_zone(vec![square(200, 300)]),
    ];
    zones[0].expanded_into = true;
    let result = EXPAND_EXPOLYGONS(&sources, &mut zones, CoordinateScale::Normal).unwrap();

    assert_eq!(
        seed_snapshots(&result.anchors),
        vec![(1, 1, SMALL_ANCHOR.to_vec()), (0, 2, LARGE_ANCHOR.to_vec())]
    );
    assert_eq!(
        expansion_snapshots(&result.expansions),
        vec![
            (1, 1, SMALL_OUTER.to_vec(), vec![SMALL_INNER.to_vec()]),
            (0, 2, LARGE_OUTER.to_vec(), vec![LARGE_INNER.to_vec()]),
        ]
    );
    assert_eq!(
        zones
            .iter()
            .map(|zone| zone.expanded_into)
            .collect::<Vec<_>>(),
        vec![false, true, true]
    );
}

#[test]
fn task22o36_both_scales_match_the_explicit_sorted_pipeline() {
    let sources = [square(200_000, 300_000)];
    let mut scale_outputs = Vec::new();
    for scale in [CoordinateScale::Normal, CoordinateScale::LargeBed] {
        let parameters = RegionExpansionParameters::build(100_000.0, 10_000.0, 5, scale);
        let mut actual_zones = vec![super::super::ExpansionZone::new(
            vec![square(0, 1_000_000)],
            parameters,
        )];
        let mut expected_zones = vec![super::super::ExpansionZone::new(
            vec![square(0, 1_000_000)],
            parameters,
        )];
        let actual = EXPAND_EXPOLYGONS(&sources, &mut actual_zones, scale).unwrap();
        let expected = explicit_pipeline(&sources, &mut expected_zones, scale).unwrap();
        assert_eq!(actual.anchors, expected.0);
        assert_eq!(actual.expansions, expected.1);
        assert_eq!(
            actual_zones[0].expanded_into,
            expected_zones[0].expanded_into
        );
        scale_outputs.push(expansion_snapshots(&actual.expansions));
    }
    assert_ne!(scale_outputs[0], scale_outputs[1]);
}

#[test]
fn task22o36_discovery_errors_preserve_failing_and_later_flags() {
    let invalid = expolygon(
        &[(OUTSIDE, 0), (OUTSIDE, 10), (OUTSIDE - 1, 10)],
        Vec::new(),
    );
    let sources = [square(20, 30)];
    let mut first = vec![expansion_zone(vec![invalid.clone()])];
    first[0].expanded_into = true;
    assert!(matches!(
        EXPAND_EXPOLYGONS(&sources, &mut first, CoordinateScale::Normal),
        Err(ClipperError::CoordinateOutOfRange)
    ));
    assert!(first[0].expanded_into);

    let mut later = vec![
        expansion_zone(vec![square(0, 100)]),
        expansion_zone(vec![invalid]),
        expansion_zone(vec![square(0, 100)]),
    ];
    later[1].expanded_into = true;
    later[2].expanded_into = true;
    assert!(matches!(
        EXPAND_EXPOLYGONS(&sources, &mut later, CoordinateScale::Normal),
        Err(ClipperError::CoordinateOutOfRange)
    ));
    assert_eq!(
        later
            .iter()
            .map(|zone| zone.expanded_into)
            .collect::<Vec<_>>(),
        vec![true, true, true]
    );
}

#[test]
fn task22o36_propagation_errors_commit_only_prior_zone_flags() {
    let huge_source = square(-1_000_000_000_000_000_000, 1_000_000_000_000_000_000);
    let huge_boundary = square(-4_000_000_000_000_000_000, 4_000_000_000_000_000_000);
    let full_expansion = 6.0e18_f32;
    let failing_parameters = RegionExpansionParameters::build(
        full_expansion,
        full_expansion,
        1,
        CoordinateScale::Normal,
    );

    let mut first = vec![super::super::ExpansionZone::new(
        vec![huge_boundary.clone()],
        failing_parameters,
    )];
    first[0].expanded_into = true;
    assert_o30_error(std::slice::from_ref(&huge_source), &first[0]);
    assert!(matches!(
        EXPAND_EXPOLYGONS(
            std::slice::from_ref(&huge_source),
            &mut first,
            CoordinateScale::Normal,
        ),
        Err(ClipperError::CoordinateOutOfRange)
    ));
    assert!(first[0].expanded_into);

    let sources = [square(20, 30), huge_source];
    let mut later = vec![
        expansion_zone(vec![square(0, 100)]),
        super::super::ExpansionZone::new(vec![huge_boundary], failing_parameters),
        expansion_zone(vec![square(0, 100)]),
    ];
    later[1].expanded_into = true;
    later[2].expanded_into = true;
    assert_o30_error(&sources, &later[1]);
    assert!(matches!(
        EXPAND_EXPOLYGONS(&sources, &mut later, CoordinateScale::Normal),
        Err(ClipperError::CoordinateOutOfRange)
    ));
    assert_eq!(
        later
            .iter()
            .map(|zone| zone.expanded_into)
            .collect::<Vec<_>>(),
        vec![true, true, true]
    );
}
