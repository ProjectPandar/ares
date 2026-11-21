//! Organic configuration and algorithms owned by `TreeSupportCommon.hpp`.

use crate::geometry::{Coord, Point, Polygon};
use std::f64::consts::TAU;

pub(super) use mesh_settings::MeshGroupSettings;
pub(super) use settings::TreeSettings;

mod area_growth;
mod branch_mesh;
mod drawing;
pub(in super::super) mod generation;
mod infill_lines;
mod initial_areas;
mod interface_placer;
mod interfaces;
mod layer_growth;
mod layer_index;
mod merge_areas;
mod mesh_settings;
mod move_inside;
mod nodes;
mod output;
mod pair_merge;
mod pathing;
mod raft;
mod rich_interface_placer;
mod roof_sampler;
mod settings;
mod single_area;
mod smoothing;
mod support_element;
mod tip_lines;
mod tip_sampling;

// TreeSupportCommon.hpp:40; Polygon.cpp:789–805 takes a chord error, not a vertex count.
const SUPPORT_TREE_CIRCLE_RESOLUTION: f64 = 25.0;

fn make_circle(radius: f64) -> Polygon {
    let angle = 2.0 * (1.0 - SUPPORT_TREE_CIRCLE_RESOLUTION / radius).acos();
    let segments = (TAU / angle).ceil() as usize;
    let increment = TAU / segments as f64;
    Polygon::new(
        (0..segments)
            .map(|index| {
                let angle = increment * index as f64;
                Point::new(
                    (angle.cos() * radius) as Coord,
                    (angle.sin() * radius) as Coord,
                )
            })
            .collect(),
    )
}
