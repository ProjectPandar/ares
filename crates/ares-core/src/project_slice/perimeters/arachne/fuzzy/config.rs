// `PerimeterGenerator.hpp::FuzzySkinConfig` and `PrintObject.cpp:3707-3708`.
use crate::{
    ProcessFuzzySkinMode, RegionOptions,
    geometry::CoordinateScale,
    perimeters::{FuzzySkinConfig, FuzzySkinKind},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::project_slice::perimeters) struct Config {
    pub(in crate::project_slice::perimeters) base: FuzzySkinConfig,
    pub(super) mode: ProcessFuzzySkinMode,
    pub(super) thickness: i64,
    pub(super) point_distance: i64,
}

impl Config {
    pub(super) fn new(region: &RegionOptions, scale: CoordinateScale) -> Self {
        let mut base = FuzzySkinConfig::from_region(region);
        if base.point_distance_mm < 0.01 || base.thickness_mm < 0.001 {
            base.kind = FuzzySkinKind::None;
        }
        let thickness = (base.thickness_mm / scale.factor()) as i64;
        let point_distance = (base.point_distance_mm / scale.factor()) as i64;
        base.thickness_mm = scale.unscale(thickness);
        base.point_distance_mm = scale.unscale(point_distance);
        Self {
            base,
            mode: region.fuzzy_skin_mode,
            thickness,
            point_distance,
        }
    }

    pub(super) fn same_effect(self, other: Self) -> bool {
        let mut left = self;
        let mut right = other;
        left.base.kind = FuzzySkinKind::None;
        right.base.kind = FuzzySkinKind::None;
        left.base.first_layer = false;
        right.base.first_layer = false;
        left == right
    }
}
