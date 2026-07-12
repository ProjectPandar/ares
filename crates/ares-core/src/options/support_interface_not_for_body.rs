use crate::{SliceError, SliceOptions};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SupportInterfaceNotForBodyOptions {
    not_for_body: bool,
}

impl SupportInterfaceNotForBodyOptions {
    pub(crate) const fn not_for_body(self) -> bool {
        self.not_for_body
    }

    pub(crate) fn consume_runtime(self) {
        let _ = self.not_for_body();
    }
}

impl SliceOptions {
    pub(crate) fn support_interface_not_for_body_options(
        &self,
    ) -> Result<SupportInterfaceNotForBodyOptions, SliceError> {
        Ok(SupportInterfaceNotForBodyOptions {
            not_for_body: self.bool_option("support_interface_not_for_body", true)?,
        })
    }
}
