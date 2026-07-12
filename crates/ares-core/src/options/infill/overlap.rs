use serde_json::Value;

use crate::{ExtrusionOptions, PrintPathRole, SliceError, SliceOptions};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct InfillWallOverlapOptions {
    infill_percent: f64,
    top_bottom_percent: f64,
}

impl InfillWallOverlapOptions {
    pub(crate) fn parse(options: &SliceOptions) -> Result<Self, SliceError> {
        Ok(Self {
            infill_percent: parse_percent_option(
                options.values().get("infill_wall_overlap"),
                "infill_wall_overlap",
                15.0,
            )?,
            top_bottom_percent: parse_percent_option(
                options.values().get("top_bottom_infill_wall_overlap"),
                "top_bottom_infill_wall_overlap",
                25.0,
            )?,
        })
    }

    pub(crate) const fn infill_percent(self) -> f64 {
        self.infill_percent
    }

    pub(crate) const fn top_bottom_percent(self) -> f64 {
        self.top_bottom_percent
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct InfillWallBoundaryOptions {
    wall_loops: u32,
    external_wall_line_width: f64,
    internal_wall_line_width: f64,
    only_one_wall_first_layer: bool,
    only_one_wall_top: bool,
    alternate_extra_wall: bool,
}

impl InfillWallBoundaryOptions {
    pub(crate) fn parse(
        options: &SliceOptions,
        extrusion_options: &ExtrusionOptions,
    ) -> Result<Self, SliceError> {
        Ok(Self {
            wall_loops: options.non_negative_u32("wall_loops", 2)?,
            external_wall_line_width: extrusion_options
                .width_for_role(PrintPathRole::ExternalPerimeter),
            internal_wall_line_width: extrusion_options
                .width_for_role(PrintPathRole::InternalPerimeter),
            only_one_wall_first_layer: options.bool_option("only_one_wall_first_layer", false)?,
            only_one_wall_top: options.bool_option("only_one_wall_top", false)?,
            alternate_extra_wall: options.bool_option("alternate_extra_wall", false)?,
        })
    }

    pub(crate) const fn wall_loops(self) -> u32 {
        self.wall_loops
    }

    pub(crate) const fn external_wall_line_width(self) -> f64 {
        self.external_wall_line_width
    }

    pub(crate) const fn internal_wall_line_width(self) -> f64 {
        self.internal_wall_line_width
    }

    pub(crate) const fn only_one_wall_first_layer(self) -> bool {
        self.only_one_wall_first_layer
    }

    pub(crate) const fn only_one_wall_top(self) -> bool {
        self.only_one_wall_top
    }

    pub(crate) const fn alternate_extra_wall(self) -> bool {
        self.alternate_extra_wall
    }

    #[cfg(test)]
    pub(crate) const fn new_for_tests(
        wall_loops: u32,
        external_wall_line_width: f64,
        internal_wall_line_width: f64,
    ) -> Self {
        Self {
            wall_loops,
            external_wall_line_width,
            internal_wall_line_width,
            only_one_wall_first_layer: false,
            only_one_wall_top: false,
            alternate_extra_wall: false,
        }
    }

    #[cfg(test)]
    pub(crate) const fn with_only_one_wall_first_layer_for_tests(self) -> Self {
        Self {
            only_one_wall_first_layer: true,
            ..self
        }
    }

    #[cfg(test)]
    pub(crate) const fn with_only_one_wall_top_for_tests(self) -> Self {
        Self {
            only_one_wall_top: true,
            ..self
        }
    }

    #[cfg(test)]
    pub(crate) const fn with_alternate_extra_wall_for_tests(self) -> Self {
        Self {
            alternate_extra_wall: true,
            ..self
        }
    }
}

fn parse_percent_option(value: Option<&Value>, key: &str, default: f64) -> Result<f64, SliceError> {
    let Some(value) = value else {
        return Ok(default);
    };
    let percent = match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => parse_percent_text(text),
        _ => None,
    }
    .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a number")))?;
    if percent.is_finite() && percent >= 0.0 {
        Ok(percent)
    } else {
        Err(SliceError::InvalidInput(format!("{key} must be non-negative")))
    }
}

fn parse_percent_text(text: &str) -> Option<f64> {
    let text = text.trim();
    text.strip_suffix('%').unwrap_or(text).trim().parse().ok()
}

#[cfg(test)]
impl InfillWallOverlapOptions {
    pub(crate) const fn new_for_tests(infill_percent: f64, top_bottom_percent: f64) -> Self {
        Self {
            infill_percent,
            top_bottom_percent,
        }
    }
}
