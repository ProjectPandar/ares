use std::collections::BTreeMap;

use crate::SliceError;

use super::super::SliceOptions;

impl SliceOptions {
    pub fn validate_infill_pattern_options(&self) -> Result<BTreeMap<String, String>, SliceError> {
        let mut errors = BTreeMap::new();

        self.validate_pattern_option(
            &mut errors,
            "sparse_infill_pattern",
            is_active_sparse_infill_pattern,
        )?;
        for key in [
            "top_surface_pattern",
            "bottom_surface_pattern",
            "internal_solid_infill_pattern",
        ] {
            self.validate_pattern_option(&mut errors, key, is_active_surface_infill_pattern)?;
        }

        Ok(errors)
    }
}

fn is_active_sparse_infill_pattern(pattern: &str) -> bool {
    matches!(
        pattern,
        "rectilinear"
            | "alignedrectilinear"
            | "zigzag"
            | "crosszag"
            | "lockedzag"
            | "line"
            | "grid"
            | "triangles"
            | "tri-hexagon"
            | "cubic"
            | "adaptivecubic"
            | "quartercubic"
            | "supportcubic"
            | "lightning"
            | "honeycomb"
            | "3dhoneycomb"
            | "lateral-honeycomb"
            | "lateral-lattice"
            | "crosshatch"
            | "tpmsd"
            | "tpmsfk"
            | "gyroid"
            | "concentric"
            | "hilbertcurve"
            | "archimedeanchords"
            | "octagramspiral"
    )
}

fn is_active_surface_infill_pattern(pattern: &str) -> bool {
    matches!(
        pattern,
        "monotonic"
            | "monotonicline"
            | "rectilinear"
            | "alignedrectilinear"
            | "concentric"
            | "hilbertcurve"
            | "archimedeanchords"
            | "octagramspiral"
    )
}
