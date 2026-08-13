use super::helpers::{expolygon, polygon, surface_snapshots};
use crate::{
    geometry::{BoundingBox, ClipperError, ExPolygon, RegionExpansionEx},
    project_slice::{
        prepare_infill::external_surfaces::{Bridge, merge_bridges::merge_bridges},
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

fn rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> ExPolygon {
    expolygon(
        &[
            (min_x, min_y),
            (max_x, min_y),
            (max_x, max_y),
            (min_x, max_y),
        ],
        Vec::new(),
    )
}

fn bridge(expolygon: ExPolygon, group_id: u32, angle: Option<f64>) -> Bridge {
    Bridge {
        expolygon,
        group_id,
        angle,
    }
}

fn expansion(expolygon: ExPolygon, src_id: u32, boundary_id: u32) -> RegionExpansionEx {
    RegionExpansionEx {
        expolygon,
        src_id,
        boundary_id,
    }
}

fn bounds(surface: &RegionSurface) -> ((i64, i64), (i64, i64)) {
    let bounds = BoundingBox::from_expolygon(surface.as_parts().1).unwrap();
    (
        (bounds.min().x(), bounds.min().y()),
        (bounds.max().x(), bounds.max().y()),
    )
}

#[test]
fn task22o40_single_bridge_with_hole_emits_exact_defaults_and_root_angle() {
    let geometry = expolygon(
        &[(0, 0), (100, 0), (100, 100), (0, 100)],
        vec![polygon(&[(30, 70), (70, 70), (70, 30), (30, 30)])],
    );
    let bridges = vec![bridge(geometry, 0, Some(0.75))];

    let surfaces = merge_bridges(bridges, &[], 5.0).unwrap();

    assert_eq!(surfaces.len(), 1);
    assert_eq!(
        surface_snapshots(&surfaces),
        vec![(
            vec![(100, 100), (0, 100), (0, 0), (100, 0)],
            vec![vec![(30, 30), (30, 70), (70, 70), (70, 30)]],
        )]
    );
    let (kind, _, thickness, thickness_layers, angle, extra_perimeters) = surfaces[0].as_parts();
    assert_eq!(kind, RegionSurfaceKind::BottomBridge);
    assert_eq!(
        (thickness, thickness_layers, angle, extra_perimeters),
        (-1.0, 1, 0.75, 0)
    );
}

#[test]
fn task22o40_empty_input_does_not_reach_closing() {
    assert!(merge_bridges(Vec::new(), &[], f32::NAN).unwrap().is_empty());
}

#[test]
fn task22o40_group_members_and_expansion_use_the_root_angle() {
    let bridges = vec![
        bridge(rectangle(0, 0, 20, 20), 0, Some(0.25)),
        bridge(rectangle(80, 0, 100, 20), 0, Some(1.25)),
    ];
    let expansions = vec![expansion(rectangle(20, 0, 80, 20), 1, 4)];

    let surfaces = merge_bridges(bridges, &expansions, 2.0).unwrap();

    assert_eq!(surfaces.len(), 1);
    assert_eq!(bounds(&surfaces[0]), ((0, 0), (100, 20)));
    let (kind, geometry, thickness, layers, angle, extra) = surfaces[0].as_parts();
    assert_eq!(kind, RegionSurfaceKind::BottomBridge);
    assert!(geometry.holes().is_empty());
    assert_eq!((thickness, layers, angle, extra), (-1.0, 1, 0.25, 0));
}

#[test]
fn task22o40_expansions_stay_with_their_sources() {
    let bridges = vec![
        bridge(rectangle(0, 0, 20, 40), 0, Some(0.5)),
        bridge(rectangle(200, 0, 220, 20), 1, Some(1.5)),
    ];
    let expansions = vec![
        expansion(rectangle(20, 0, 40, 40), 0, 7),
        expansion(
            expolygon(
                &[(40, 0), (100, 0), (100, 40), (40, 40)],
                vec![polygon(&[(60, 30), (80, 30), (80, 10), (60, 10)])],
            ),
            0,
            8,
        ),
        expansion(rectangle(220, 0, 260, 20), 1, 2),
    ];

    let surfaces = merge_bridges(bridges, &expansions, 2.0).unwrap();

    assert_eq!(surfaces.len(), 2);
    assert_eq!(bounds(&surfaces[0]), ((0, 0), (100, 40)));
    assert_eq!(bounds(&surfaces[1]), ((200, 0), (260, 20)));
    assert_eq!(
        surfaces[0].as_parts().1.holes()[0].points(),
        &[
            crate::geometry::Point::new(60, 10),
            crate::geometry::Point::new(60, 30),
            crate::geometry::Point::new(80, 30),
            crate::geometry::Point::new(80, 10),
        ]
    );
    assert_eq!(surfaces[0].as_parts().4, 0.5);
    assert_eq!(surfaces[1].as_parts().4, 1.5);
}

#[test]
fn task22o40_same_group_narrow_gap_is_closed() {
    let bridges = vec![
        bridge(rectangle(0, 0, 20, 20), 0, Some(0.375)),
        bridge(rectangle(27, 0, 47, 20), 0, None),
    ];

    let surfaces = merge_bridges(bridges, &[], 5.0).unwrap();

    assert_eq!(
        surface_snapshots(&surfaces),
        vec![(vec![(47, 20), (0, 20), (0, 0), (47, 0)], Vec::new(),)]
    );
    assert_eq!(surfaces[0].as_parts().4, 0.375);
}

fn disconnected_group() -> Vec<Bridge> {
    vec![
        bridge(rectangle(0, 0, 20, 20), 0, Some(0.125)),
        bridge(rectangle(100, 0, 120, 20), 0, None),
    ]
}

#[test]
fn task22o40_disconnected_group_output_is_deterministic() {
    let first = merge_bridges(disconnected_group(), &[], 2.0).unwrap();
    let second = merge_bridges(disconnected_group(), &[], 2.0).unwrap();

    assert_eq!(first.len(), 2);
    assert_eq!(surface_snapshots(&first), surface_snapshots(&second));
    assert!(first.iter().all(|surface| {
        let (kind, _, _, _, angle, _) = surface.as_parts();
        kind == RegionSurfaceKind::BottomBridge && angle == 0.125
    }));
}

#[test]
fn task22o40_closes_groups_independently_with_contours_and_holes() {
    let bridges = vec![
        bridge(
            expolygon(
                &[(0, 0), (100, 0), (100, 100), (0, 100)],
                vec![polygon(&[(30, 70), (70, 70), (70, 30), (30, 30)])],
            ),
            0,
            Some(0.25),
        ),
        bridge(rectangle(130, 0, 180, 50), 0, Some(9.0)),
        bridge(rectangle(187, 0, 237, 50), 2, Some(1.25)),
    ];

    let surfaces = merge_bridges(bridges, &[], 5.0).unwrap();

    assert_eq!(
        surface_snapshots(&surfaces),
        vec![
            (
                vec![(100, 100), (0, 100), (0, 0), (100, 0)],
                vec![vec![(30, 30), (30, 70), (70, 70), (70, 30)]],
            ),
            (vec![(180, 50), (130, 50), (130, 0), (180, 0)], vec![]),
            (vec![(237, 50), (187, 50), (187, 0), (237, 0)], vec![]),
        ]
    );
    assert_eq!(
        surfaces
            .iter()
            .map(|surface| surface.as_parts().4)
            .collect::<Vec<_>>(),
        vec![0.25, 0.25, 1.25]
    );
}

#[test]
fn task22o40_propagates_clipper_coordinate_errors() {
    let invalid = rectangle(i64::MAX - 10, 0, i64::MAX, 10);

    assert!(matches!(
        merge_bridges(vec![bridge(invalid, 0, Some(0.5))], &[], 2.0),
        Err(ClipperError::CoordinateOutOfRange)
    ));
}
