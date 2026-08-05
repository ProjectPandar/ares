use crate::{
    geometry::CoordinateScale,
    project_slice::prepare_infill::{
        vertical_shell_filtering::{
            self, GeometryStep,
            filter::{RecordOperands, filter_record},
        },
        vertical_shell_regularization::types::VerticalShellRegularization,
        vertical_shell_trimming::types::VerticalShellTrim,
    },
};

use super::{empty_record, rectangle};

#[test]
fn task22o23_empty_o21_trim_short_circuits_all_filtering_geometry() {
    vertical_shell_filtering::reset_geometry_hooks();
    let output = filter_record(
        RecordOperands {
            trim: &VerticalShellTrim { shell: Vec::new() },
            regularization: &VerticalShellRegularization {
                regularized_shell: Vec::new(),
            },
            current: &empty_record(),
            previous_lslices: None,
            next_lslices: None,
        },
        20,
        CoordinateScale::Normal,
    )
    .unwrap();
    assert!(output.filtered_shell.is_empty());
    assert!(vertical_shell_filtering::geometry_events().is_empty());
}

#[test]
fn task22o23_empty_o22_output_still_builds_both_volumes_and_reaches_empty_gate() {
    vertical_shell_filtering::reset_geometry_hooks();
    let output = filter_record(
        RecordOperands {
            trim: &VerticalShellTrim {
                shell: vec![rectangle(0, 0, 1, 1)],
            },
            regularization: &VerticalShellRegularization {
                regularized_shell: Vec::new(),
            },
            current: &empty_record(),
            previous_lslices: None,
            next_lslices: None,
        },
        20,
        CoordinateScale::Normal,
    )
    .unwrap();
    assert!(output.filtered_shell.is_empty());
    assert_eq!(
        vertical_shell_filtering::geometry_events(),
        vec![
            GeometryStep::NeighborIntersection,
            GeometryStep::ClosingGrow,
            GeometryStep::ClosingShrink,
            GeometryStep::EmptyGate,
        ]
    );
}
