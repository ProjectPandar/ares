use super::*;
use crate::{
    BrimPath, InfillPath, InfillRole, LayerGapFills, LayerInfills, LayerPerimeters, PerimeterPath,
    PerimeterRole, Point2, SkirtPath,
};

pub(super) fn sample_skirts(layer_id: usize, print_z: f64) -> Vec<LayerSkirts> {
    vec![LayerSkirts::new(layer_id, print_z, Vec::new())]
}

pub(super) fn sample_brims(layer_id: usize, print_z: f64) -> Vec<LayerBrims> {
    vec![LayerBrims::new(layer_id, print_z, Vec::new())]
}

pub(super) fn sample_non_empty_brims(layer_id: usize, print_z: f64) -> Vec<LayerBrims> {
    vec![LayerBrims::new(
        layer_id,
        print_z,
        vec![
            BrimPath::new(vec![
                Point2::new(-2.0, -2.0),
                Point2::new(3.0, -2.0),
                Point2::new(3.0, 3.0),
                Point2::new(-2.0, 3.0),
            ])
            .unwrap(),
        ],
    )]
}

pub(super) fn sample_non_empty_skirts(layer_id: usize, print_z: f64) -> Vec<LayerSkirts> {
    vec![LayerSkirts::new(
        layer_id,
        print_z,
        vec![
            SkirtPath::new(vec![
                Point2::new(-1.0, -1.0),
                Point2::new(2.0, -1.0),
                Point2::new(2.0, 2.0),
                Point2::new(-1.0, 2.0),
            ])
            .unwrap(),
        ],
    )]
}

pub(super) fn sample_perimeters(layer_id: usize, print_z: f64) -> Vec<LayerPerimeters> {
    vec![LayerPerimeters::new(
        layer_id,
        print_z,
        vec![
            PerimeterPath::new(
                PerimeterRole::External,
                vec![
                    Point2::new(0.0, 0.0),
                    Point2::new(1.0, 0.0),
                    Point2::new(1.0, 1.0),
                ],
            )
            .unwrap(),
        ],
    )]
}

pub(super) fn sample_internal_perimeters(layer_id: usize, print_z: f64) -> Vec<LayerPerimeters> {
    vec![LayerPerimeters::new(
        layer_id,
        print_z,
        vec![
            PerimeterPath::new(
                PerimeterRole::Internal,
                vec![
                    Point2::new(0.4, 0.4),
                    Point2::new(3.6, 0.4),
                    Point2::new(3.6, 3.6),
                    Point2::new(0.4, 3.6),
                ],
            )
            .unwrap(),
        ],
    )]
}

pub(super) fn sample_infills(layer_id: usize, print_z: f64) -> Vec<LayerInfills> {
    vec![LayerInfills::new(
        layer_id,
        print_z,
        vec![
            InfillPath::new(
                InfillRole::Sparse,
                vec![Point2::new(0.5, 0.0), Point2::new(0.5, 1.0)],
                0.2,
            )
            .unwrap(),
        ],
    )]
}

pub(super) fn sample_gap_fills(layer_id: usize, print_z: f64) -> [LayerGapFills; 1] {
    [LayerGapFills::new(layer_id, print_z, Vec::new())]
}
