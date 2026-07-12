use serde_json::Value;

use crate::{SliceError, SliceOptions};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GapFillTarget {
    Everywhere,
    TopBottom,
    Nowhere,
}

impl GapFillTarget {
    pub(crate) const fn allows_top_bottom(self) -> bool {
        matches!(self, Self::Everywhere | Self::TopBottom)
    }

    pub(crate) const fn allows_internal_solid(self) -> bool {
        matches!(self, Self::Everywhere)
    }

    #[cfg(test)]
    pub(crate) const fn as_str_for_tests(self) -> &'static str {
        match self {
            Self::Everywhere => "everywhere",
            Self::TopBottom => "topbottom",
            Self::Nowhere => "nowhere",
        }
    }
}

pub(crate) fn parse_gap_fill_target(options: &SliceOptions) -> Result<GapFillTarget, SliceError> {
    parse_target(options.values().get("gap_fill_target"))
}

pub(crate) fn parse_filter_out_gap_fill(options: &SliceOptions) -> Result<f64, SliceError> {
    options.range_f64("filter_out_gap_fill", 0.0, f64::NEG_INFINITY, f64::INFINITY)
}

fn parse_target(value: Option<&Value>) -> Result<GapFillTarget, SliceError> {
    let Some(value) = value else {
        return Ok(GapFillTarget::Nowhere);
    };
    match value.as_str() {
        Some("everywhere") => Ok(GapFillTarget::Everywhere),
        Some("topbottom") => Ok(GapFillTarget::TopBottom),
        Some("nowhere") => Ok(GapFillTarget::Nowhere),
        Some(_) => Err(SliceError::InvalidInput(
            "gap_fill_target has unknown enum value".to_owned(),
        )),
        None => Err(SliceError::InvalidInput(
            "gap_fill_target must be a string".to_owned(),
        )),
    }
}
