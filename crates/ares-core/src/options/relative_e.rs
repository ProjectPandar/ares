use super::SliceOptions;
use crate::SliceError;

impl SliceOptions {
    pub(crate) fn use_relative_e_distances(&self) -> Result<bool, SliceError> {
        self.bool_option("use_relative_e_distances", true)
    }
}
