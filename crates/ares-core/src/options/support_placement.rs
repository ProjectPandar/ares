use crate::{SliceError, SliceOptions};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SupportPlacementOptions {
    object_xy_distance_mm: f64,
    object_first_layer_gap_mm: f64,
    on_build_plate_only: bool,
    critical_regions_only: bool,
    remove_small_overhang: bool,
}

impl SupportPlacementOptions {
    pub(crate) const fn object_xy_distance_mm(self) -> f64 {
        self.object_xy_distance_mm
    }

    pub(crate) const fn object_first_layer_gap_mm(self) -> f64 {
        self.object_first_layer_gap_mm
    }

    pub(crate) const fn on_build_plate_only(self) -> bool {
        self.on_build_plate_only
    }

    pub(crate) const fn critical_regions_only(self) -> bool {
        self.critical_regions_only
    }

    pub(crate) const fn remove_small_overhang(self) -> bool {
        self.remove_small_overhang
    }

    pub(crate) fn consume_runtime(self) {
        let _ = (
            self.object_xy_distance_mm(),
            self.object_first_layer_gap_mm(),
            self.on_build_plate_only(),
            self.critical_regions_only(),
            self.remove_small_overhang(),
        );
    }
}

impl SliceOptions {
    pub(crate) fn support_placement_options(&self) -> Result<SupportPlacementOptions, SliceError> {
        Ok(SupportPlacementOptions {
            object_xy_distance_mm: self.range_f64("support_object_xy_distance", 0.35, 0.0, 10.0)?,
            object_first_layer_gap_mm: self.range_f64(
                "support_object_first_layer_gap",
                0.2,
                0.0,
                10.0,
            )?,
            on_build_plate_only: self.bool_option("support_on_build_plate_only", false)?,
            critical_regions_only: self.bool_option("support_critical_regions_only", false)?,
            remove_small_overhang: self.bool_option("support_remove_small_overhang", true)?,
        })
    }
}
