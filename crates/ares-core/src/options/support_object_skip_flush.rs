use crate::{SliceError, SliceOptions};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SupportObjectSkipFlushOptions {
    skip_flush: bool,
}

impl SupportObjectSkipFlushOptions {
    pub(crate) const fn skip_flush(self) -> bool {
        self.skip_flush
    }

    pub(crate) fn consume_runtime(self) {
        let _ = self.skip_flush();
    }
}

impl SliceOptions {
    pub(crate) fn support_object_skip_flush_options(
        &self,
    ) -> Result<SupportObjectSkipFlushOptions, SliceError> {
        Ok(SupportObjectSkipFlushOptions {
            skip_flush: self.bool_option("support_object_skip_flush", false)?,
        })
    }
}
