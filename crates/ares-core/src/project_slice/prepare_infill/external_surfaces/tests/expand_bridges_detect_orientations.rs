use super::{
    expand_expolygons, get_grouped_bridges,
    helpers::{expolygon, snapshots, square, surface, surface_snapshots, zone},
};
use crate::{
    geometry::{ClipperError, CoordinateScale, ExPolygon, difference_ex},
    project_slice::{
        prepare_infill::external_surfaces::{
            ExpansionZone, detect_bridge_directions::detect_bridge_directions,
            expand_bridges_detect_orientations::expand_bridges_detect_orientations,
            merge_bridges::merge_bridges,
        },
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

const OUTSIDE: i64 = 0x4000_0000_0000_0000;

fn explicit_sorted_pipeline(
    sources: Vec<ExPolygon>,
    zones: &mut [ExpansionZone],
) -> Vec<RegionSurface> {
    let mut expanded = expand_expolygons(&sources, zones, CoordinateScale::Normal).unwrap();
    assert_eq!(
        expanded
            .anchors
            .iter()
            .map(|anchor| (anchor.src, anchor.boundary))
            .collect::<Vec<_>>(),
        vec![(1, 1), (0, 2)]
    );
    assert_eq!(
        expanded
            .expansions
            .iter()
            .map(|expansion| (expansion.src_id, expansion.boundary_id))
            .collect::<Vec<_>>(),
        vec![(1, 1), (0, 2)]
    );
    let mut bridges = get_grouped_bridges(sources, &expanded.expansions).unwrap();
    expanded
        .anchors
        .sort_by_key(|anchor| (anchor.src, anchor.boundary));
    detect_bridge_directions(
        &expanded.anchors,
        &mut bridges,
        zones,
        CoordinateScale::Normal,
    )
    .unwrap();
    expanded
        .expansions
        .sort_by_key(|expansion| (expansion.src_id, expansion.boundary_id));
    let output = merge_bridges(bridges, &expanded.expansions, 1.0).unwrap();
    let clips = output
        .iter()
        .map(|surface| surface.as_parts().1.clone())
        .collect::<Vec<_>>();
    for zone in zones.iter_mut().filter(|zone| zone.expanded_into) {
        zone.expolygons = difference_ex(&zone.expolygons, &clips).unwrap();
    }
    output
}

#[test]
fn task22o41_expands_and_orients_a_bottom_bridge() {
    let mut surfaces = vec![surface(
        RegionSurfaceKind::BottomBridge,
        square(20, 30),
        (0.2, 1, -1.0, 0),
    )];
    let untouched = square(200, 300);
    let mut zones = vec![zone(vec![square(0, 100)]), zone(vec![untouched.clone()])];

    let output =
        expand_bridges_detect_orientations(&mut surfaces, &mut zones, 1.0, CoordinateScale::Normal)
            .unwrap();

    assert!(!output.is_empty());
    assert!(output.iter().all(|surface| {
        let (kind, _, thickness, thickness_layers, bridge_angle, extra_perimeters) =
            surface.as_parts();
        kind == RegionSurfaceKind::BottomBridge
            && thickness == -1.0
            && thickness_layers == 1
            && bridge_angle >= 0.0
            && extra_perimeters == 0
    }));
    assert!(surface_snapshots(&surfaces)[0].0.is_empty());
    assert!(zones[0].expanded_into);
    assert!(!zones[1].expanded_into);
    assert_eq!(snapshots(&zones[1].expolygons), snapshots(&[untouched]));
}

#[test]
fn task22o41_without_bottom_bridges_is_an_exact_noop() {
    let original = square(20, 30);
    let boundary = square(0, 100);
    let mut surfaces = vec![surface(
        RegionSurfaceKind::Top,
        original.clone(),
        (0.2, 2, 1.5, 3),
    )];
    let mut zones = vec![zone(vec![boundary.clone()])];

    let output =
        expand_bridges_detect_orientations(&mut surfaces, &mut zones, 0.0, CoordinateScale::Normal)
            .unwrap();

    assert!(output.is_empty());
    assert_eq!(surface_snapshots(&surfaces), snapshots(&[original]));
    assert_eq!(snapshots(&zones[0].expolygons), snapshots(&[boundary]));
    assert!(!zones[0].expanded_into);
    assert_eq!(surfaces[0].as_parts().0, RegionSurfaceKind::Top);
}

#[test]
fn task22o41_preserves_nonmatching_surfaces_while_moving_bridge_geometry() {
    let top = square(200, 210);
    let mut surfaces = vec![
        surface(RegionSurfaceKind::Top, top.clone(), (0.2, 1, -1.0, 0)),
        surface(
            RegionSurfaceKind::BottomBridge,
            square(20, 30),
            (0.3, 2, 0.5, 4),
        ),
    ];
    let mut zones = vec![zone(vec![square(0, 100)])];

    let output =
        expand_bridges_detect_orientations(&mut surfaces, &mut zones, 1.0, CoordinateScale::Normal)
            .unwrap();

    assert!(!output.is_empty());
    assert_eq!(surface_snapshots(&surfaces[..1]), snapshots(&[top]));
    assert!(surface_snapshots(&surfaces[1..])[0].0.is_empty());
}

#[test]
fn task22o41_sorts_zone_major_results_before_direction_and_merge() {
    let sources = vec![square(220, 230), square(20, 30)];
    let make_zones = || {
        vec![
            zone(vec![square(400, 500)]),
            zone(vec![square(0, 100)]),
            zone(vec![square(200, 300)]),
        ]
    };
    let mut expected_zones = make_zones();
    let expected = explicit_sorted_pipeline(sources.clone(), &mut expected_zones);
    let mut actual_zones = make_zones();
    let mut surfaces = sources
        .into_iter()
        .map(|source| surface(RegionSurfaceKind::BottomBridge, source, (0.2, 1, -1.0, 0)))
        .collect::<Vec<_>>();

    let actual = expand_bridges_detect_orientations(
        &mut surfaces,
        &mut actual_zones,
        1.0,
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(surface_snapshots(&actual), surface_snapshots(&expected));
    assert_eq!(
        actual
            .iter()
            .map(|surface| surface.as_parts().4.to_bits())
            .collect::<Vec<_>>(),
        expected
            .iter()
            .map(|surface| surface.as_parts().4.to_bits())
            .collect::<Vec<_>>()
    );
    for (actual, expected) in actual_zones.iter().zip(expected_zones) {
        assert_eq!(actual.expanded_into, expected.expanded_into);
        assert_eq!(
            snapshots(&actual.expolygons),
            snapshots(&expected.expolygons)
        );
    }
}

#[test]
fn task22o41_first_expansion_error_follows_bridge_extraction() {
    let invalid = expolygon(
        &[(OUTSIDE, 0), (OUTSIDE, 10), (OUTSIDE - 1, 10)],
        Vec::new(),
    );
    let mut surfaces = vec![surface(
        RegionSurfaceKind::BottomBridge,
        square(20, 30),
        (0.2, 1, -1.0, 0),
    )];
    let mut zones = vec![zone(vec![invalid])];

    assert!(matches!(
        expand_bridges_detect_orientations(&mut surfaces, &mut zones, 1.0, CoordinateScale::Normal,),
        Err(ClipperError::CoordinateOutOfRange)
    ));
    assert!(surface_snapshots(&surfaces)[0].0.is_empty());
    assert!(!zones[0].expanded_into);
}

#[test]
fn task22o41_later_expansion_error_keeps_prior_flag_without_final_trim() {
    let first = square(0, 100);
    let first_snapshot = snapshots(std::slice::from_ref(&first));
    let invalid = expolygon(
        &[(OUTSIDE, 0), (OUTSIDE, 10), (OUTSIDE - 1, 10)],
        Vec::new(),
    );
    let mut surfaces = vec![surface(
        RegionSurfaceKind::BottomBridge,
        square(20, 30),
        (0.2, 1, -1.0, 0),
    )];
    let mut zones = vec![zone(vec![first]), zone(vec![invalid])];

    assert!(matches!(
        expand_bridges_detect_orientations(&mut surfaces, &mut zones, 1.0, CoordinateScale::Normal,),
        Err(ClipperError::CoordinateOutOfRange)
    ));
    assert!(surface_snapshots(&surfaces)[0].0.is_empty());
    assert!(zones[0].expanded_into);
    assert!(!zones[1].expanded_into);
    assert_eq!(snapshots(&zones[0].expolygons), first_snapshot);
}
