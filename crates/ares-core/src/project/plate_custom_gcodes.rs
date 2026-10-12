//! The project's per-plate custom G-codes (`Model::plates_custom_gcodes`,
//! `Model.hpp:1562-1570`), loaded from `Metadata/custom_gcode_per_layer.xml`
//! (`bbs_3mf.cpp:220`, `:3208-3292`).
use std::collections::{BTreeMap, BTreeSet};

use crate::{SliceError, gcode_processor::CustomGCodeItem};

use super::{PackagePath, ProjectArchive, domain::Project};

/// `CustomGCode::Mode` (`CustomGCode.hpp:68-76`).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CustomGCodeMode {
    #[default]
    Undef,
    SingleExtruder,
    MultiAsSingle,
    MultiExtruder,
}

/// `CustomGCode::Info` (`CustomGCode.hpp:82-92`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CustomGCodeInfo {
    pub mode: CustomGCodeMode,
    pub gcodes: Vec<CustomGCodeItem>,
}

pub(crate) fn load(
    _archive: &mut ProjectArchive<'_>,
    _archive_paths: &BTreeSet<PackagePath>,
) -> Result<BTreeMap<i32, CustomGCodeInfo>, SliceError> {
    Ok(BTreeMap::new())
}

impl Project {
    /// `Model::plates_custom_gcodes`, keyed by the zero-based plate index.
    pub fn plates_custom_gcodes(&self) -> &BTreeMap<i32, CustomGCodeInfo> {
        &self.plates_custom_gcodes
    }

    /// `Model::get_curr_plate_custom_gcodes` (`Model.hpp:1565-1570`): the
    /// selected plate's entry, or an empty `Info`. An unselected project has
    /// `curr_plate_index` 0, as upstream right after loading.
    pub fn curr_plate_custom_gcodes(&self) -> CustomGCodeInfo {
        self.plates_custom_gcodes
            .get(&self.curr_plate_index)
            .cloned()
            .unwrap_or_default()
    }
}
