use crate::{SliceError, SliceOptions};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SupportEnableOptions {
    enabled: bool,
}

impl SupportEnableOptions {
    pub(crate) const fn enabled(self) -> bool {
        self.enabled
    }

    pub(crate) fn consume_runtime(self) {
        let _ = self.enabled();
    }
}

impl SliceOptions {
    pub(crate) fn support_enable_options(&self) -> Result<SupportEnableOptions, SliceError> {
        Ok(SupportEnableOptions {
            enabled: self.bool_option("enable_support", false)?,
        })
    }
}
