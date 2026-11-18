//! `TreeSupport3D.cpp:656-738::generate_support_infill_lines`.

use crate::fill::honeycomb::{HoneycombFillParams, fill_honeycomb};
use crate::fill::rectilinear::{MonotonicFillParams, fill_rectilinear_surface};
use crate::geometry::{
    BoundingBox, ClipperError, CoordinateScale, FillRule, JoinType, Polygon, Polyline,
    offset_expolygon, union_ex,
};
use crate::project_slice::raft::{
    fill_params::RaftFillPattern,
    fills::support_base::{SupportBaseFillParams, fill_surface},
};
use crate::project_slice::support_material::toolpath_parameters::ToolpathParameters;

pub(super) struct InfillLayer {
    pub(super) roof: bool,
    pub(super) layer_index: usize,
    pub(super) support_infill_distance: i64,
}

pub(super) fn generate(
    polygons: &[Polygon],
    parameters: &ToolpathParameters,
    layer: InfillLayer,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let InfillLayer {
        roof,
        layer_index,
        support_infill_distance,
    } = layer;
    let flow = if roof {
        parameters.interface_flow
    } else {
        parameters.base_flow
    };
    let pattern = if roof {
        parameters.interface_fill_pattern
    } else {
        parameters.base_fill_pattern
    };
    // Beta adds the parity angle to the interface angle in f32.
    let angle = if roof {
        parameters.interface_angle
            + if layer_index & 1 != 0 {
                -std::f32::consts::FRAC_PI_4
            } else {
                std::f32::consts::FRAC_PI_4
            }
    } else {
        parameters.base_angle
    };
    let spacing = f64::from(flow.spacing);
    let scaled_spacing = (spacing / scale.factor()) as f32;
    let density = if roof {
        parameters.top_interface_density as f32
    } else {
        scaled_spacing / (scaled_spacing + support_infill_distance as f32)
    };
    let mut out = Vec::new();
    for expolygon in union_ex(polygons, FillRule::NonZero)? {
        let reference = BoundingBox::from_polygon(expolygon.contour())
            .expect("union component has a contour")
            .center();
        match pattern {
            RaftFillPattern::SupportBase => out.extend(fill_surface(
                &expolygon,
                SupportBaseFillParams {
                    spacing,
                    density,
                    angle,
                    reference,
                    reference_angle: angle + std::f32::consts::FRAC_PI_2,
                    outer_offset: (-0.5 * spacing / scale.factor()) as f32,
                },
                scale,
            )?),
            RaftFillPattern::Rectilinear => {
                let params = MonotonicFillParams {
                    dont_alternate_fill_direction: false,
                    layer_angle_fixed: false,
                    spacing,
                    overlap: 0.0,
                    density,
                    angle,
                    layer_index,
                    thickness_layers: 1,
                    fixed_angle: false,
                    bridge_angle: None,
                    reference_point: reference,
                    dont_adjust: true,
                    anchor_length_max: 1000.0,
                    link_max_length: 0.0,
                    consistent_pattern: false,
                    horiz_move: 0.0,
                    extended_object_min_x: None,
                    extended_object_max_x: None,
                    locked_zag: false,
                };
                out.extend(fill_rectilinear_surface(&expolygon, params, scale)?.polylines);
            }
            RaftFillPattern::Honeycomb => {
                let params = HoneycombFillParams {
                    dont_alternate_fill_direction: false,
                    spacing,
                    density,
                    multiline: 1,
                    smooth_factor: 0.0,
                    resolution: 0.0125,
                    angle,
                    layer_index,
                    thickness_layers: 1,
                    fixed_angle: false,
                    bridge_angle: None,
                    anchor_length: 1000.0,
                    anchor_length_max: 1000.0,
                };
                for component in offset_expolygon(
                    &expolygon,
                    (-0.5 * spacing / scale.factor()) as f32,
                    JoinType::Miter,
                    3.0,
                )? {
                    out.extend(fill_honeycomb(&component, params, scale)?);
                }
            }
            RaftFillPattern::Grid | RaftFillPattern::Concentric | RaftFillPattern::SpiralInset => {
                unreachable!("SupportParameters selects contact patterns separately")
            }
        }
    }
    Ok(out)
}
