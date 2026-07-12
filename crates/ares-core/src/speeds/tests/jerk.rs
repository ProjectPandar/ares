use super::*;

#[test]
fn jerk_options_apply_orca_role_precedence() {
    let jerk = JerkOptions {
        initial_layer_mm_s: 0.0,
        ..configured_jerk_options()
    };

    assert_eq!(
        jerk.jerk_for_layer(
            ToolpathMoveKind::Travel,
            PrintPathRole::ExternalPerimeter,
            false
        ),
        Some(11.0)
    );
    assert_eq!(
        jerk.jerk_for_layer(
            ToolpathMoveKind::Print,
            PrintPathRole::ExternalPerimeter,
            false
        ),
        Some(7.0)
    );
    assert_eq!(
        jerk.jerk_for_layer(
            ToolpathMoveKind::Print,
            PrintPathRole::InternalPerimeter,
            false
        ),
        Some(4.0)
    );
    assert_eq!(
        jerk.jerk_for_layer(ToolpathMoveKind::Print, PrintPathRole::SparseInfill, false),
        Some(5.0)
    );
    assert_eq!(
        jerk.jerk_for_layer(ToolpathMoveKind::Print, PrintPathRole::Bridge, false),
        Some(5.0)
    );
}

#[test]
fn first_layer_jerk_overrides_print_roles_only_when_positive() {
    let jerk = configured_jerk_options();

    assert_eq!(
        jerk.jerk_for_layer(
            ToolpathMoveKind::Travel,
            PrintPathRole::ExternalPerimeter,
            true
        ),
        Some(5.5)
    );
    assert_eq!(
        jerk.jerk_for_layer(
            ToolpathMoveKind::Print,
            PrintPathRole::InternalPerimeter,
            true
        ),
        Some(6.0)
    );

    let disabled_initial = JerkOptions {
        initial_layer_mm_s: 0.0,
        ..configured_jerk_options()
    };
    assert_eq!(
        disabled_initial.jerk_for_layer(
            ToolpathMoveKind::Print,
            PrintPathRole::InternalPerimeter,
            true
        ),
        Some(4.0)
    );
}

#[test]
fn default_zero_disables_jerk_and_travel_zero_has_no_fallback() {
    let disabled = JerkOptions {
        default_mm_s: 0.0,
        ..configured_jerk_options()
    };
    assert_eq!(
        disabled.jerk_for_layer(
            ToolpathMoveKind::Print,
            PrintPathRole::ExternalPerimeter,
            false
        ),
        None
    );
    assert_eq!(
        disabled.jerk_for_layer(
            ToolpathMoveKind::Travel,
            PrintPathRole::ExternalPerimeter,
            false
        ),
        None
    );

    let no_travel = JerkOptions {
        travel_mm_s: 0.0,
        initial_layer_travel_mm_s: 0.0,
        ..configured_jerk_options()
    };
    assert_eq!(
        no_travel.jerk_for_layer(
            ToolpathMoveKind::Travel,
            PrintPathRole::ExternalPerimeter,
            false
        ),
        None
    );
}

fn configured_jerk_options() -> JerkOptions {
    JerkOptions {
        default_mm_s: 8.0,
        initial_layer_mm_s: 6.0,
        outer_wall_mm_s: 7.0,
        inner_wall_mm_s: 4.0,
        infill_mm_s: 5.0,
        top_surface_mm_s: 5.0,
        travel_mm_s: 11.0,
        initial_layer_travel_mm_s: 5.5,
    }
}
