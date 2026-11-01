use serde::{Deserialize, Serialize};

/// PrintConfig.hpp:238-242 and PrintConfig.cpp:256-261 (345efec64c7b).
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArcFittingMode {
    #[default]
    Disabled,
    Enabled,
    SparseInfill,
}

impl ArcFittingMode {
    /// LayerRegion.cpp:1104-1112: only internal infill is sparse.
    pub(crate) const fn fits(self, is_sparse_infill: bool) -> bool {
        match self {
            Self::Disabled => false,
            Self::Enabled => true,
            Self::SparseInfill => is_sparse_infill,
        }
    }
}
