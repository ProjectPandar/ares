//! `convert_entities` thin-wall classification tests — `VariableWidth.cpp:225-230`
//! wraps each converted run whose `first_point() == last_point()` into an
//! `ExtrusionLoop` and emits open runs as bare `ExtrusionPath`s.

use super::convert_entities;
use crate::{
    geometry::{CoordinateScale, Point, Polyline, to_thick_polylines},
    project_slice::perimeters::{
        classic::{entity_collections::ExtrusionEntity, materialize::ExtrusionRole},
        types::Flow,
    },
};

fn flow() -> Flow {
    Flow {
        width: 0.45,
        height: 0.2,
        spacing: 0.45,
        nozzle_diameter: 0.4,
        bridge: false,
        mm3_per_mm: 0.09,
    }
}

fn thick(points: &[(i64, i64)], width: f64) -> crate::geometry::ThickPolyline {
    to_thick_polylines(
        vec![Polyline::new(
            points.iter().map(|&(x, y)| Point::new(x, y)).collect(),
        )],
        width,
    )
    .remove(0)
}

#[test]
fn open_run_emits_bare_paths() {
    // A straight open thin wall -> open `ExtrusionPath`s, never a loop.
    let thin = thick(&[(0, 0), (1_000_000, 0)], 0.4e6);
    let entities = convert_entities(
        std::slice::from_ref(&thin),
        flow(),
        CoordinateScale::Normal,
        ExtrusionRole::ExternalPerimeter,
    )
    .unwrap();
    assert!(!entities.is_empty());
    assert!(
        entities
            .iter()
            .all(|e| matches!(e, ExtrusionEntity::Path(_))),
        "an open thin wall must not produce an ExtrusionLoop"
    );
}

#[test]
fn closed_run_wraps_as_loop() {
    // A closed thin ring (first == last point) wraps into `ExtrusionLoop` so it
    // keeps loop seam/clipping/orientation behavior (VariableWidth.cpp:226-229).
    let thin = thick(
        &[(0, 0), (1_000_000, 0), (1_000_000, 1_000_000), (0, 0)],
        0.4e6,
    );
    let entities = convert_entities(
        std::slice::from_ref(&thin),
        flow(),
        CoordinateScale::Normal,
        ExtrusionRole::ExternalPerimeter,
    )
    .unwrap();
    assert_eq!(entities.len(), 1);
    match &entities[0] {
        ExtrusionEntity::Loop(ordered) => {
            assert_eq!(ordered.inset_idx, -1, "thin-wall loop inset_idx stays -1");
            assert!(!ordered.extrusion_loop.paths.is_empty());
        }
        other => panic!("a closed thin ring must produce a Loop, got {other:?}"),
    }
}
