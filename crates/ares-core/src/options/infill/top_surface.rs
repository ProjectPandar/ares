use serde_json::Value;

use crate::{PrintPathRole, SliceError};

pub(super) fn parse_min_width_top_surface(
    options: &super::super::SliceOptions,
) -> Result<f64, SliceError> {
    let base = options
        .extrusion_options()?
        .width_for_role(PrintPathRole::InternalPerimeter);
    match options.values().get("min_width_top_surface") {
        Some(value) => crate::options::parsing::parse_non_negative_numeric_or_percent_over_base(
            "min_width_top_surface",
            value,
            base,
        ),
        None => crate::options::parsing::parse_non_negative_numeric_or_percent_over_base(
            "min_width_top_surface",
            &Value::String("300%".to_owned()),
            base,
        ),
    }
}
