use crate::{
    SliceError,
    project_slice::prepare_infill::{
        vertical_shell_regularization::{
            GeometryStep, fail_geometry_at, geometry_events, regularize, reset_geometry_hooks,
        },
        vertical_shell_trimming::types::VerticalShellTrim,
    },
};

#[test]
fn task22o22_every_geometry_site_maps_to_the_stable_error_at_its_exact_boundary() {
    let trim = VerticalShellTrim {
        shell: vec![super::rectangle(0, 0, 4_000_000, 4_000_000)],
    };
    for (step, expected_events) in [
        (GeometryStep::Union, vec![GeometryStep::Union]),
        (
            GeometryStep::Offset2First,
            vec![GeometryStep::Union, GeometryStep::Offset2First],
        ),
        (
            GeometryStep::Offset2Second,
            vec![
                GeometryStep::Union,
                GeometryStep::Offset2First,
                GeometryStep::Offset2Second,
            ],
        ),
        (
            GeometryStep::Shrink,
            vec![
                GeometryStep::Union,
                GeometryStep::Offset2First,
                GeometryStep::Offset2Second,
                GeometryStep::Shrink,
            ],
        ),
    ] {
        reset_geometry_hooks();
        fail_geometry_at(step);
        assert_eq!(
            regularize::regularize_record(&trim, 400_000).unwrap_err(),
            SliceError::InvalidInput(
                "vertical-shell regularization geometry is outside the supported Clipper range"
                    .to_owned()
            )
        );
        assert_eq!(geometry_events(), expected_events);
    }
    reset_geometry_hooks();
}
