use super::*;
use crate::{InfillPath, InfillRole, LayerInfills, Point2, SliceError};

mod bridge_no_support;
mod gap_fill_filter;
mod gap_fill_order;
mod overhang;
mod shell_thickness;
mod solid_surface_roles;
mod support;

use support::*;

#[test]
fn defaults_to_perimeters_before_infills() {
    let output = generate_print_paths(
        PrintPathInput::new(
            &sample_skirts(0, 0.2),
            &sample_brims(0, 0.2),
            &sample_perimeters(0, 0.2),
            &sample_gap_fills(0, 0.2),
            &sample_infills(0, 0.2),
        ),
        ShellLayerOptions::new(1, 1),
        false,
        false,
    )
    .unwrap();

    assert_eq!(
        output[0].paths()[0].role(),
        PrintPathRole::ExternalPerimeter
    );
    assert_eq!(output[0].paths()[1].role(), PrintPathRole::SparseInfill);
}

#[test]
fn keeps_first_layer_wall_first_when_infill_first_is_enabled() {
    let output = generate_print_paths(
        PrintPathInput::new(
            &sample_skirts(0, 0.2),
            &sample_brims(0, 0.2),
            &sample_perimeters(0, 0.2),
            &sample_gap_fills(0, 0.2),
            &sample_infills(0, 0.2),
        ),
        ShellLayerOptions::new(1, 1),
        true,
        false,
    )
    .unwrap();

    assert_eq!(
        output[0].paths()[0].role(),
        PrintPathRole::ExternalPerimeter
    );
    assert_eq!(output[0].paths()[1].role(), PrintPathRole::SparseInfill);
}

#[test]
fn supports_infill_first_ordering_after_first_layer() {
    let output = generate_print_paths(
        PrintPathInput::new(
            &sample_skirts(1, 0.4),
            &sample_brims(1, 0.4),
            &sample_perimeters(1, 0.4),
            &sample_gap_fills(1, 0.4),
            &sample_infills(1, 0.4),
        ),
        ShellLayerOptions::new(1, 1),
        true,
        false,
    )
    .unwrap();

    assert_eq!(output[0].paths()[0].role(), PrintPathRole::SparseInfill);
    assert_eq!(
        output[0].paths()[1].role(),
        PrintPathRole::ExternalPerimeter
    );
}

#[test]
fn maps_internal_perimeters_to_internal_print_path_role() {
    let output = generate_print_paths(
        PrintPathInput::new(
            &sample_skirts(0, 0.2),
            &sample_brims(0, 0.2),
            &sample_internal_perimeters(0, 0.2),
            &sample_gap_fills(0, 0.2),
            &sample_infills(0, 0.2),
        ),
        ShellLayerOptions::new(1, 1),
        false,
        false,
    )
    .unwrap();

    assert_eq!(
        output[0].paths()[0].role(),
        PrintPathRole::InternalPerimeter
    );
    assert_eq!(
        PrintPathRole::InternalPerimeter.as_str(),
        "internal_perimeter"
    );
    assert_eq!(PrintPathRole::InternalBridge.as_str(), "internal_bridge");
    assert_eq!(PrintPathRole::GapFill.as_str(), "gap_fill");
}

#[test]
fn prepends_skirts_before_existing_path_order() {
    let output = generate_print_paths(
        PrintPathInput::new(
            &sample_non_empty_skirts(1, 0.4),
            &sample_brims(1, 0.4),
            &sample_perimeters(1, 0.4),
            &sample_gap_fills(1, 0.4),
            &sample_infills(1, 0.4),
        ),
        ShellLayerOptions::new(1, 1),
        true,
        false,
    )
    .unwrap();

    assert_eq!(output[0].paths()[0].role(), PrintPathRole::Skirt);
    assert_eq!(output[0].paths()[1].role(), PrintPathRole::SparseInfill);
    assert_eq!(
        output[0].paths()[2].role(),
        PrintPathRole::ExternalPerimeter
    );
}

#[test]
fn orders_skirts_then_brims_before_existing_path_order() {
    let output = generate_print_paths(
        PrintPathInput::new(
            &sample_non_empty_skirts(1, 0.4),
            &sample_non_empty_brims(1, 0.4),
            &sample_perimeters(1, 0.4),
            &sample_gap_fills(1, 0.4),
            &sample_infills(1, 0.4),
        ),
        ShellLayerOptions::new(1, 1),
        true,
        false,
    )
    .unwrap();

    assert_eq!(output[0].paths()[0].role(), PrintPathRole::Skirt);
    assert_eq!(output[0].paths()[1].role(), PrintPathRole::Brim);
    assert_eq!(output[0].paths()[2].role(), PrintPathRole::SparseInfill);
    assert_eq!(
        output[0].paths()[3].role(),
        PrintPathRole::ExternalPerimeter
    );
}

#[test]
fn preserves_layer_metadata_and_points() {
    let output = generate_print_paths(
        PrintPathInput::new(
            &sample_skirts(3, 0.6),
            &sample_brims(3, 0.6),
            &sample_perimeters(3, 0.6),
            &sample_gap_fills(3, 0.6),
            &sample_infills(3, 0.6),
        ),
        ShellLayerOptions::new(1, 1),
        false,
        false,
    )
    .unwrap();

    assert_eq!(output[0].layer_id(), 3);
    assert_eq!(output[0].print_z(), 0.6);
    assert_eq!(
        output[0].paths()[0].points(),
        &[
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 0.0),
            Point2::new(1.0, 1.0),
        ]
    );
    assert_eq!(
        output[0].paths()[1].points(),
        &[Point2::new(0.5, 0.0), Point2::new(0.5, 1.0)]
    );
}

#[test]
fn copies_sparse_infill_effective_height_to_print_path() {
    let infills = vec![LayerInfills::new(
        0,
        0.2,
        vec![
            InfillPath::new(
                InfillRole::Sparse,
                vec![Point2::new(0.5, 0.0), Point2::new(0.5, 1.0)],
                0.4,
            )
            .unwrap(),
        ],
    )];

    let output = generate_print_paths(
        PrintPathInput::new(
            &sample_skirts(0, 0.2),
            &sample_brims(0, 0.2),
            &sample_perimeters(0, 0.2),
            &sample_gap_fills(0, 0.2),
            &infills,
        ),
        ShellLayerOptions::new(1, 1),
        false,
        false,
    )
    .unwrap();

    assert_eq!(output[0].paths()[1].effective_layer_height_mm(), Some(0.4));
    assert_eq!(output[0].paths()[0].effective_layer_height_mm(), None);
}

#[test]
fn maps_single_layer_solid_infill_to_bottom_surface_print_path_role() {
    let infills = vec![LayerInfills::new(
        0,
        0.2,
        vec![
            InfillPath::new(
                InfillRole::Solid,
                vec![Point2::new(0.5, 0.0), Point2::new(0.5, 1.0)],
                0.2,
            )
            .unwrap(),
        ],
    )];

    let output = generate_print_paths(
        PrintPathInput::new(
            &sample_skirts(0, 0.2),
            &sample_brims(0, 0.2),
            &sample_perimeters(0, 0.2),
            &sample_gap_fills(0, 0.2),
            &infills,
        ),
        ShellLayerOptions::new(1, 1),
        false,
        false,
    )
    .unwrap();

    assert_eq!(output[0].paths()[1].role(), PrintPathRole::BottomSurface);
    assert_eq!(PrintPathRole::BottomSurface.as_str(), "bottom_surface");
}

#[test]
fn rejects_mismatched_layers() {
    assert!(matches!(
        generate_print_paths(
            PrintPathInput::new(
                &[],
                &sample_brims(0, 0.2),
                &sample_perimeters(0, 0.2),
                &sample_gap_fills(0, 0.2),
                &sample_infills(0, 0.2),
            ),
            ShellLayerOptions::new(1, 1),
            false,
            false,
        ),
        Err(SliceError::InvalidInput(_))
    ));
    assert!(matches!(
        generate_print_paths(
            PrintPathInput::new(
                &sample_skirts(1, 0.2),
                &sample_brims(1, 0.2),
                &sample_perimeters(1, 0.2),
                &sample_gap_fills(1, 0.2),
                &sample_infills(2, 0.2),
            ),
            ShellLayerOptions::new(1, 1),
            false,
            false,
        ),
        Err(SliceError::InvalidInput(_))
    ));
    assert!(matches!(
        generate_print_paths(
            PrintPathInput::new(
                &sample_skirts(1, 0.2),
                &sample_brims(1, 0.2),
                &sample_perimeters(1, 0.2),
                &sample_gap_fills(1, 0.2),
                &sample_infills(1, 0.4),
            ),
            ShellLayerOptions::new(1, 1),
            false,
            false,
        ),
        Err(SliceError::InvalidInput(_))
    ));
}
