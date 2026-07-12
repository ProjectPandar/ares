use crate::{SliceError, SliceOptions, options::support_type::SupportType};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SupportStyle {
    Default,
    Grid,
    Snug,
    TreeOrganic,
    TreeSlim,
    TreeStrong,
    TreeHybrid,
}

impl SupportStyle {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) const fn is_tree_style(self) -> bool {
        matches!(
            self,
            Self::TreeOrganic | Self::TreeSlim | Self::TreeStrong | Self::TreeHybrid
        )
    }

    pub(crate) const fn is_snug(self) -> bool {
        matches!(self, Self::Snug)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn resolve_for_support_type(self, support_type: SupportType) -> Self {
        let is_tree = support_type.is_tree();
        let style = match (self, is_tree) {
            (Self::Grid | Self::Snug, true) => Self::Default,
            (
                Self::TreeOrganic | Self::TreeSlim | Self::TreeStrong | Self::TreeHybrid,
                false,
            ) => Self::Default,
            (style, _) => style,
        };

        match (style, is_tree) {
            (Self::Default, true) => Self::TreeOrganic,
            (Self::Default, false) => Self::Grid,
            (style, _) => style,
        }
    }
}

impl SliceOptions {
    pub(crate) fn support_style(&self) -> Result<SupportStyle, SliceError> {
        parse(self)
    }
}

pub(crate) fn parse(options: &SliceOptions) -> Result<SupportStyle, SliceError> {
    let Some(value) = options.values.get("support_style") else {
        return Ok(SupportStyle::Default);
    };
    let Some(value) = value.as_str() else {
        return Err(SliceError::InvalidInput(
            "support_style must be a string".to_owned(),
        ));
    };

    match value {
        "default" => Ok(SupportStyle::Default),
        "grid" => Ok(SupportStyle::Grid),
        "snug" => Ok(SupportStyle::Snug),
        "organic" => Ok(SupportStyle::TreeOrganic),
        "tree_slim" => Ok(SupportStyle::TreeSlim),
        "tree_strong" => Ok(SupportStyle::TreeStrong),
        "tree_hybrid" => Ok(SupportStyle::TreeHybrid),
        _ => Err(SliceError::InvalidInput(format!(
            "support_style contains invalid value {value}"
        ))),
    }
}
