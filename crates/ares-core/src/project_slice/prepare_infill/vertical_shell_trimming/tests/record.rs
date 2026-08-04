use crate::{
    SliceError,
    geometry::{
        ExPolygon, difference_polygons_paths, intersection_polygons_paths_with_safety_offset,
    },
    project_slice::{
        prepare_infill::{
            vertical_shell_projection::types::VerticalShellProjection,
            vertical_shell_trimming::{
                GeometryStep, fail_geometry_at, geometry_events, reset_geometry_hooks, trim,
            },
        },
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

#[test]
fn task22o21_inactive_record_is_some_empty_and_has_zero_events() {
    let record = super::record(vec![super::surface(RegionSurfaceKind::Internal, 0, 40)]);
    let projection = VerticalShellProjection {
        shell: vec![super::square(0, 40)],
        holes: Vec::new(),
    };
    reset_geometry_hooks();
    let output = trim::trim_record(&record, &projection, false).unwrap();
    assert!(output.shell.is_empty());
    assert!(geometry_events().is_empty());
}

#[test]
fn task22o21_empty_projected_shell_can_gain_internal_difference() {
    let record = super::record(vec![super::surface(RegionSurfaceKind::Internal, 0, 40)]);
    let projection = VerticalShellProjection {
        shell: Vec::new(),
        holes: Vec::new(),
    };
    reset_geometry_hooks();
    let output = trim::trim_record(&record, &projection, true).unwrap();
    assert!(!output.shell.is_empty());
    assert_eq!(
        geometry_events(),
        vec![
            GeometryStep::SafetyOffset,
            GeometryStep::SafetyIntersection,
            GeometryStep::Difference,
            GeometryStep::EmptyGate,
            GeometryStep::SolidAppend,
        ]
    );
}

#[test]
fn task22o21_solid_participates_then_is_appended_verbatim_after_nonempty_gate() {
    let solid = super::square(60, 90);
    let record = super::record(vec![
        super::surface(RegionSurfaceKind::Internal, 0, 40),
        RegionSurface::new(
            RegionSurfaceKind::InternalSolid,
            ExPolygon::new(solid.clone(), Vec::new()),
        ),
    ]);
    let projection = VerticalShellProjection {
        shell: Vec::new(),
        holes: Vec::new(),
    };
    reset_geometry_hooks();
    let output = trim::trim_record(&record, &projection, true).unwrap();
    assert_eq!(output.shell.last(), Some(&solid));
    assert_eq!(
        geometry_events(),
        vec![
            GeometryStep::SafetyOffset,
            GeometryStep::SafetyIntersection,
            GeometryStep::Difference,
            GeometryStep::EmptyGate,
            GeometryStep::SolidAppend,
        ]
    );
}

#[test]
fn task22o21_output_is_intersection_then_difference_then_verbatim_solid() {
    let internal = super::square(0, 40);
    let solid = super::square(60, 90);
    let record = super::record(vec![
        RegionSurface::new(
            RegionSurfaceKind::Internal,
            ExPolygon::new(internal.clone(), Vec::new()),
        ),
        RegionSurface::new(
            RegionSurfaceKind::InternalSolid,
            ExPolygon::new(solid.clone(), Vec::new()),
        ),
    ]);
    let projection = VerticalShellProjection {
        shell: vec![super::square(20, 70)],
        holes: vec![super::square(10, 30)],
    };
    let internal_paths = vec![internal, solid.clone()];
    let mut expected =
        intersection_polygons_paths_with_safety_offset(&projection.shell, &internal_paths).unwrap();
    expected.extend(difference_polygons_paths(&internal_paths, &projection.holes).unwrap());
    expected.push(solid);
    reset_geometry_hooks();
    assert_eq!(
        trim::trim_record(&record, &projection, true).unwrap().shell,
        expected
    );
}

#[test]
fn task22o21_complete_erasure_takes_gate_before_present_solid_append() {
    let solid = super::square(0, 40);
    let record = super::record(vec![RegionSurface::new(
        RegionSurfaceKind::InternalSolid,
        ExPolygon::new(solid.clone(), Vec::new()),
    )]);
    let projection = VerticalShellProjection {
        shell: vec![super::square(100, 120)],
        holes: vec![solid],
    };
    reset_geometry_hooks();
    let output = trim::trim_record(&record, &projection, true).unwrap();
    assert!(output.shell.is_empty());
    assert_eq!(geometry_events().last(), Some(&GeometryStep::EmptyGate));
    assert!(!geometry_events().contains(&GeometryStep::SolidAppend));
}

#[test]
fn task22o21_empty_internal_input_still_runs_both_flat_boolean_sites() {
    let record = super::record(Vec::new());
    let projection = VerticalShellProjection {
        shell: vec![super::square(0, 40)],
        holes: Vec::new(),
    };
    reset_geometry_hooks();
    let output = trim::trim_record(&record, &projection, true).unwrap();
    assert!(output.shell.is_empty());
    assert_eq!(
        geometry_events(),
        vec![
            GeometryStep::SafetyOffset,
            GeometryStep::SafetyIntersection,
            GeometryStep::Difference,
            GeometryStep::EmptyGate,
        ]
    );
}

#[test]
fn task22o21_each_geometry_site_uses_the_stable_boundary_error() {
    let record = super::record(vec![super::surface(RegionSurfaceKind::Internal, 0, 40)]);
    let projection = VerticalShellProjection {
        shell: vec![super::square(0, 40)],
        holes: Vec::new(),
    };
    for step in [
        GeometryStep::SafetyOffset,
        GeometryStep::SafetyIntersection,
        GeometryStep::Difference,
    ] {
        reset_geometry_hooks();
        fail_geometry_at(step);
        assert_eq!(
            trim::trim_record(&record, &projection, true).unwrap_err(),
            SliceError::InvalidInput(
                "vertical-shell internal trimming geometry is outside the supported Clipper range"
                    .to_owned()
            )
        );
    }
    reset_geometry_hooks();
}
