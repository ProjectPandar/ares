use super::{expand_merge_surfaces, helpers::*};
use crate::{
    geometry::{CoordinateScale, RegionExpansionParameters},
    project_slice::region_slices::RegionSurfaceKind,
};

fn run_pipeline_case(
    sources: Vec<crate::geometry::ExPolygon>,
    actual_zones: &mut [super::super::ExpansionZone],
    expected_zones: &mut [super::super::ExpansionZone],
    closing_radius: f32,
    scale: CoordinateScale,
) {
    let expected =
        explicit_pipeline(sources.clone(), expected_zones, closing_radius, scale).unwrap();
    let mut surfaces = sources
        .into_iter()
        .map(|source| surface(RegionSurfaceKind::Bottom, source, (0.2, 1, -1.0, 0)))
        .collect::<Vec<_>>();
    let actual = expand_merge_surfaces(
        &mut surfaces,
        RegionSurfaceKind::Bottom,
        actual_zones,
        closing_radius,
        -1.0,
        scale,
    )
    .unwrap();

    assert_eq!(surface_snapshots(&actual), snapshots(&expected));
    assert!(!actual.is_empty());
    assert!(surfaces.iter().all(|surface| {
        let geometry = surface.as_parts().1;
        geometry.contour().points().is_empty() && geometry.holes().is_empty()
    }));
    assert_eq!(actual_zones.len(), expected_zones.len());
    for (actual, expected) in actual_zones.iter().zip(expected_zones) {
        assert_eq!(actual.expanded_into, expected.expanded_into);
        assert_eq!(
            snapshots(&actual.expolygons),
            snapshots(&expected.expolygons)
        );
    }
}

#[test]
fn task22o35_multiple_ordered_zones_match_the_complete_explicit_pipeline() {
    let source = vec![expolygon(
        &[(20, 20), (30, 20), (30, 30), (20, 30)],
        vec![polygon(&[(23, 23), (23, 27), (27, 27), (27, 23)])],
    )];
    let mut actual_zones = vec![zone(vec![square(0, 100)]), zone(vec![square(200, 300)])];
    let mut expected_zones = vec![zone(vec![square(0, 100)]), zone(vec![square(200, 300)])];

    run_pipeline_case(
        source,
        &mut actual_zones,
        &mut expected_zones,
        1.0,
        CoordinateScale::Normal,
    );

    assert!(actual_zones[0].expanded_into);
    assert!(!actual_zones[1].expanded_into);
}

#[test]
fn task22o35_multiple_sources_and_boundaries_preserve_complete_topology_and_order() {
    let sources = vec![
        expolygon(
            &[(220, 220), (230, 220), (230, 230), (220, 230)],
            vec![polygon(&[(223, 223), (223, 227), (227, 227), (227, 223)])],
        ),
        square(20, 30),
    ];
    let boundaries = vec![square(0, 100), square(200, 300)];
    let mut actual_zones = vec![zone(boundaries.clone())];
    let mut expected_zones = vec![zone(boundaries)];

    run_pipeline_case(
        sources,
        &mut actual_zones,
        &mut expected_zones,
        1.0,
        CoordinateScale::Normal,
    );
    assert!(actual_zones[0].expanded_into);
}

fn dual_scale_case(scale: CoordinateScale) -> Vec<ExPolygonSnapshot> {
    let params = RegionExpansionParameters::build(100_000.0, 10_000.0, 5, scale);
    let mut actual_zones = vec![super::super::ExpansionZone::new(
        vec![square(0, 1_000_000)],
        params,
    )];
    let mut expected_zones = vec![super::super::ExpansionZone::new(
        vec![square(0, 1_000_000)],
        params,
    )];
    let sources = vec![square(200_000, 300_000)];
    let expected =
        explicit_pipeline(sources.clone(), &mut expected_zones, 10_000.0, scale).unwrap();
    let mut surfaces = vec![surface(
        RegionSurfaceKind::Top,
        sources.into_iter().next().unwrap(),
        (0.2, 1, -1.0, 0),
    )];
    let actual = expand_merge_surfaces(
        &mut surfaces,
        RegionSurfaceKind::Top,
        &mut actual_zones,
        10_000.0,
        -1.0,
        scale,
    )
    .unwrap();
    assert_eq!(surface_snapshots(&actual), snapshots(&expected));
    assert_eq!(
        snapshots(&actual_zones[0].expolygons),
        snapshots(&expected_zones[0].expolygons)
    );
    surface_snapshots(&actual)
}

#[test]
fn task22o35_normal_and_large_bed_vectors_match_their_explicit_pipelines() {
    let normal = dual_scale_case(CoordinateScale::Normal);
    let large_bed = dual_scale_case(CoordinateScale::LargeBed);
    assert!(!normal.is_empty());
    assert!(!large_bed.is_empty());
    assert_ne!(normal, large_bed);
}
