use crate::{
    SliceError,
    project_slice::prepare_infill::vertical_shell_projection::{
        GeometryStep, fail_geometry_at, gather, reset_geometry_hooks,
    },
};

#[test]
fn task22o20_hole_and_shell_boolean_failures_use_the_same_exact_error() {
    for step in [GeometryStep::HoleIntersection, GeometryStep::ShellUnion] {
        reset_geometry_hooks();
        fail_geometry_at(step);
        let mut current = vec![super::square(0, 20)];
        let next = [super::square(10, 30)];
        let error = if step == GeometryStep::HoleIntersection {
            gather::combine_holes(&mut current, &next).unwrap_err()
        } else {
            gather::combine_shells(&mut current, &next).unwrap_err()
        };
        assert_eq!(
            error,
            SliceError::InvalidInput(
                "vertical-shell projection geometry is outside the supported Clipper range"
                    .to_owned()
            )
        );
    }
    reset_geometry_hooks();
}
