//! Enabled smart-overhang path in `TreeSupport.cpp::detect_overhangs`.

mod clusters;
pub(super) mod finalize;
mod first_pass;
mod propagation;

use super::super::{layer_input::LayerInput, overhangs::overlaps};
use crate::{
    CancelToken, ObjectOptions, SliceError,
    geometry::{ClipperError, CoordinateScale, ExPolygon, JoinType, intersection_ex, offset2_ex},
    parallel::par_range_map_ordered_try,
};

pub(super) struct OverhangLayer {
    pub(super) extrudable: Vec<ExPolygon>,
    pub(super) raw_overhangs: Vec<ExPolygon>,
    pub(super) overhangs: Vec<ExPolygon>,
    pub(super) overhang_types: Vec<OverhangType>,
    pub(super) sharp_tails: Vec<ExPolygon>,
    pub(super) sharp_tail_heights: Vec<f32>,
    pub(super) cantilevers: Vec<ExPolygon>,
    height: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum OverhangType {
    Detected,
    Enforced,
    SharpTail,
}

pub(super) struct SmartOverhangs<'a> {
    pub(super) automatic: bool,
    object: &'a ObjectOptions,
    scale: CoordinateScale,
    extrusion_width: f64,
    extrusion_width_scaled: f64,
    threshold_radians: f64,
    pub(super) layers: Vec<OverhangLayer>,
    pub(super) detect_sharp_tails: bool,
    pub(super) remove_small_overhangs: bool,
    pub(super) has_sharp_tails: bool,
    pub(super) has_cantilever: bool,
    pub(super) max_cantilever_distance: f64,
    pub(super) highest_overhang_layer: usize,
}

impl<'a> SmartOverhangs<'a> {
    pub(super) fn new(
        object: &'a ObjectOptions,
        layers: &[LayerInput<'_>],
        (extrusion_width, scale): (f64, CoordinateScale),
        cancel: &CancelToken,
        geometry_error: fn(ClipperError) -> SliceError,
    ) -> Result<Self, SliceError> {
        let extrusion_width_scaled = scale.scaled_delta(extrusion_width);
        // TreeSupport.cpp:880–892 prepares each physical layer before detection.
        let prepared = par_range_map_ordered_try(layers.len(), cancel, |index| {
            let layer = &layers[index];
            let opened = offset2_ex(
                layer.slices,
                (-extrusion_width_scaled / 2.0) as f32,
                extrusion_width_scaled as f32,
                JoinType::Miter,
                3.0,
            )
            .map_err(geometry_error)?;
            Ok(OverhangLayer {
                extrudable: intersection_ex(layer.slices, &opened).map_err(geometry_error)?,
                raw_overhangs: Vec::new(),
                overhangs: Vec::new(),
                overhang_types: Vec::new(),
                sharp_tails: Vec::new(),
                sharp_tail_heights: Vec::new(),
                cantilevers: Vec::new(),
                height: layer.height,
            })
        })?;
        let threshold = if object.support_threshold_angle.0 > 0 {
            f64::from(object.support_threshold_angle.0) + 1.0
        } else {
            30.0
        }
        .min(89.0);
        Ok(Self {
            automatic: matches!(object.support_type, crate::ProcessSupportType::TreeAuto),
            object,
            scale,
            extrusion_width,
            extrusion_width_scaled,
            threshold_radians: std::f64::consts::PI * threshold / 180.0,
            layers: prepared,
            detect_sharp_tails: true,
            remove_small_overhangs: object.support_remove_small_overhang.0,
            has_sharp_tails: false,
            has_cantilever: false,
            max_cantilever_distance: 0.0,
            highest_overhang_layer: 0,
        })
    }
}
