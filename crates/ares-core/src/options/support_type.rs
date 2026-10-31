use crate::{SliceError, SliceOptions};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SupportType {
    NormalAuto,
    TreeAuto,
    NormalManual,
    TreeManual,
}

impl SupportType {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) const fn is_tree(self) -> bool {
        matches!(self, Self::TreeAuto | Self::TreeManual)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) const fn is_auto(self) -> bool {
        matches!(self, Self::NormalAuto | Self::TreeAuto)
    }

}

pub(crate) fn parse(options: &SliceOptions) -> Result<SupportType, SliceError> {
    let Some(value) = options.values.get("support_type") else {
        return Ok(SupportType::NormalAuto);
    };
    let Some(value) = value.as_str() else {
        return Err(SliceError::InvalidInput(
            "support_type must be a string".to_owned(),
        ));
    };

    match value {
        "normal(auto)" => Ok(SupportType::NormalAuto),
        "tree(auto)" => Ok(SupportType::TreeAuto),
        "normal(manual)" => Ok(SupportType::NormalManual),
        "tree(manual)" => Ok(SupportType::TreeManual),
        _ => Err(SliceError::InvalidInput(format!(
            "support_type contains invalid value {value}"
        ))),
    }
}
