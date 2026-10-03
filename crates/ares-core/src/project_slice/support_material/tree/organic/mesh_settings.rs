//! `TreeSupportMeshGroupSettings` constructor (`TreeSupportCommon.hpp:36-98`).

use std::f64::consts::{FRAC_PI_2, PI};

use crate::{
    ObjectOptions, OrcaFloats, RegionOptions, SliceError,
    geometry::{Coord, CoordinateScale},
    project_slice::{
        parameters::SlicingParameters,
        perimeters::flow::{SupportFlows, resolve_nominal_external_perimeter_flow},
    },
};

const EPSILON: f64 = 1e-4;

pub(in super::super) struct MeshGroupInput<'a> {
    pub(in super::super) object: &'a ObjectOptions,
    pub(in super::super) nominal_flows: &'a SupportFlows,
    pub(in super::super) regions: &'a [RegionOptions],
    pub(in super::super) nozzle_diameters: &'a OrcaFloats,
    pub(in super::super) resolution: f64,
    pub(in super::super) slicing: &'a SlicingParameters,
    pub(in super::super) scale: CoordinateScale,
}

pub(in super::super) struct MeshGroupSettings {
    pub(in super::super) layer_height: Coord,
    pub(in super::super) resolution: Coord,
    pub(in super::super) min_feature_size: Coord,
    pub(in super::super) support_line_width: Coord,
    pub(in super::super) buildplate_only: bool,
    pub(in super::super) xy_distance: Coord,
    pub(in super::super) overhang_xy_distance: Coord,
    pub(in super::super) top_distance: Coord,
    pub(in super::super) bottom_distance: Coord,
    pub(in super::super) roof_layers: i32,
    pub(in super::super) floor_layers: i32,
    pub(in super::super) roof_line_distance: Coord,
    pub(in super::super) minimum_roof_area: f64,
    pub(in super::super) minimum_support_area: Coord,
    pub(in super::super) support_offset: Coord,
    pub(in super::super) branch_distance: Coord,
    pub(in super::super) branch_angle: f64,
    pub(in super::super) slow_angle: f64,
    pub(in super::super) branch_diameter: Coord,
    pub(in super::super) diameter_angle: f64,
    pub(in super::super) top_rate: f64,
    pub(in super::super) tip_diameter: Coord,
    pub(in super::super) max_diameter_increase_to_model: Coord,
    pub(in super::super) min_height_to_model: Coord,
    pub(in super::super) buildplate_diameter: Coord,
}

impl MeshGroupSettings {
    pub(in super::super) fn new(input: MeshGroupInput<'_>) -> Result<Self, SliceError> {
        let MeshGroupInput {
            object,
            nominal_flows,
            regions,
            nozzle_diameters,
            resolution,
            slicing,
            scale,
        } = input;
        let mut maximum_nominal_external_width = 0.0_f64;
        for region in regions {
            let flow = resolve_nominal_external_perimeter_flow(object, region, nozzle_diameters)?;
            maximum_nominal_external_width =
                maximum_nominal_external_width.max(f64::from(flow.width));
        }
        let scaled = |value| scale.scaled_delta(value) as Coord;
        let layer_height = scaled(object.layer_height.0);
        let support_line_width = scaled(f64::from(nominal_flows.base.width));
        let support_roof_line_width = scaled(f64::from(nominal_flows.interface.width));
        let bottom_layers = if object.support_interface_bottom_layers.0 < 0 {
            object.support_interface_top_layers.0
        } else {
            object.support_interface_bottom_layers.0
        };
        let xy_distance = scaled(object.support_object_xy_distance.0);
        let branch_angle = (object.tree_support_branch_angle_organic.0 * PI / 180.0)
            .clamp(0.0, FRAC_PI_2 - EPSILON);
        // The official 2.4.2 binary applies max(value, 0), then min with
        // branch_angle-EPSILON, including the source's inverted zero-angle bound.
        let slow_angle = (object.tree_support_angle_slow.0 * PI / 180.0)
            .max(0.0)
            .min(branch_angle - EPSILON);
        let branch_diameter = scaled(object.tree_support_branch_diameter_organic.0);
        Ok(Self {
            layer_height,
            resolution: scaled(resolution),
            min_feature_size: scaled(object.min_feature_size.0),
            support_line_width,
            buildplate_only: object.support_on_build_plate_only.0,
            xy_distance,
            overhang_xy_distance: xy_distance.min(scaled(0.5 * maximum_nominal_external_width)),
            top_distance: scaled(slicing.gap_support_object),
            bottom_distance: scaled(slicing.gap_object_support),
            roof_layers: object.support_interface_top_layers.0,
            floor_layers: bottom_layers,
            roof_line_distance: scaled(object.support_interface_spacing.0)
                + support_roof_line_width,
            minimum_roof_area: scale.scaled_delta(scale.scaled_delta(1.0)),
            minimum_support_area: 0,
            support_offset: 0,
            branch_distance: scaled(object.tree_support_branch_distance_organic.0),
            branch_angle,
            slow_angle,
            branch_diameter,
            diameter_angle: (object.tree_support_branch_diameter_angle.0 * PI / 180.0)
                .clamp(0.0, FRAC_PI_2 - EPSILON),
            top_rate: object.tree_support_top_rate.0,
            tip_diameter: scaled(object.tree_support_tip_diameter.0).clamp(0, branch_diameter),
            max_diameter_increase_to_model: scaled(1.0),
            min_height_to_model: scaled(1.0),
            buildplate_diameter: scaled(7.5),
        })
    }
}
