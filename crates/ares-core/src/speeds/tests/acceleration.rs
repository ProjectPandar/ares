use super::*;

#[test]
fn acceleration_options_apply_orca_role_precedence() {
    let acceleration = AccelerationOptions {
        initial_layer_mm_s2: 0.0,
        ..configured_acceleration_options()
    };

    assert_eq!(
        acceleration.acceleration_for_layer(
            ToolpathMoveKind::Travel,
            PrintPathRole::ExternalPerimeter,
            false
        ),
        Some(900.0)
    );
    assert_eq!(
        acceleration.acceleration_for_layer(
            ToolpathMoveKind::Print,
            PrintPathRole::ExternalPerimeter,
            false
        ),
        Some(450.0)
    );
    assert_eq!(
        acceleration.acceleration_for_layer(
            ToolpathMoveKind::Print,
            PrintPathRole::InternalPerimeter,
            false
        ),
        Some(650.0)
    );
    assert_eq!(
        acceleration.acceleration_for_layer(
            ToolpathMoveKind::Print,
            PrintPathRole::SparseInfill,
            false
        ),
        Some(350.0)
    );
    assert_eq!(
        acceleration.acceleration_for_layer(ToolpathMoveKind::Print, PrintPathRole::Bridge, false),
        Some(225.0)
    );
}

#[test]
fn first_layer_acceleration_overrides_print_roles_only_when_positive() {
    let acceleration = AccelerationOptions {
        initial_layer_travel_mm_s2: 420.0,
        ..configured_acceleration_options()
    };

    assert_eq!(
        acceleration.acceleration_for_layer(
            ToolpathMoveKind::Travel,
            PrintPathRole::ExternalPerimeter,
            true
        ),
        Some(420.0)
    );
    assert_eq!(
        acceleration.acceleration_for_layer(
            ToolpathMoveKind::Print,
            PrintPathRole::InternalPerimeter,
            true
        ),
        Some(250.0)
    );

    let disabled_initial = AccelerationOptions {
        initial_layer_mm_s2: 0.0,
        ..configured_acceleration_options()
    };
    assert_eq!(
        disabled_initial.acceleration_for_layer(
            ToolpathMoveKind::Print,
            PrintPathRole::InternalPerimeter,
            true
        ),
        Some(650.0)
    );
}

#[test]
fn default_zero_disables_acceleration_and_travel_zero_has_no_fallback() {
    let disabled = AccelerationOptions {
        default_mm_s2: 0.0,
        ..configured_acceleration_options()
    };
    assert_eq!(
        disabled.acceleration_for_layer(
            ToolpathMoveKind::Print,
            PrintPathRole::ExternalPerimeter,
            false
        ),
        None
    );
    assert_eq!(
        disabled.acceleration_for_layer(
            ToolpathMoveKind::Travel,
            PrintPathRole::ExternalPerimeter,
            false
        ),
        None
    );

    let no_travel = AccelerationOptions {
        travel_mm_s2: 0.0,
        ..configured_acceleration_options()
    };
    assert_eq!(
        no_travel.acceleration_for_layer(
            ToolpathMoveKind::Travel,
            PrintPathRole::ExternalPerimeter,
            false
        ),
        None
    );
}

fn configured_acceleration_options() -> AccelerationOptions {
    AccelerationOptions {
        default_mm_s2: 700.0,
        initial_layer_mm_s2: 250.0,
        outer_wall_mm_s2: 450.0,
        bridge_mm_s2: 225.0,
        inner_wall_mm_s2: 650.0,
        travel_mm_s2: 900.0,
        initial_layer_travel_mm_s2: 900.0,
        sparse_infill_mm_s2: 350.0,
        internal_solid_infill_mm_s2: 350.0,
        top_surface_mm_s2: 700.0,
    }
}
