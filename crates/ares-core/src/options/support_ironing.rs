use crate::{SliceError, SliceOptions};

pub(crate) fn parse_support_ironing(options: &SliceOptions) -> Result<bool, SliceError> {
    options.bool_option("support_ironing", false)
}
