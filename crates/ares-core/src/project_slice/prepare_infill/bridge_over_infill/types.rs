use std::collections::BTreeMap;

use crate::geometry::Polygon;
use crate::project_slice::region_slices::RegionSurface;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) struct CandidateSource {
    pub(in crate::project_slice) layer_index: usize,
    pub(in crate::project_slice) region_index: usize,
    pub(in crate::project_slice) surface_index: usize,
}

pub(in crate::project_slice) struct CandidateSurface {
    pub(in crate::project_slice) source: CandidateSource,
    pub(in crate::project_slice) new_polygons: Vec<Polygon>,
    pub(in crate::project_slice) bridge_angle: f64,
}

pub(in crate::project_slice) struct BridgeCandidateObject {
    pub(in crate::project_slice) has_lightning_infill: bool,
    pub(in crate::project_slice) surfaces_by_layer: BTreeMap<usize, Vec<CandidateSurface>>,
    /// `PrintObject.cpp:2593` LIGHTNING INFILL SECTION — the expanded
    /// `fill_surfaces` (stInternal→stInternalSolid near candidates) per layer,
    /// used to build the lightning `Generator` for both anchoring and emit.
    /// Empty for non-lightning objects.
    pub(in crate::project_slice) lightning_expanded_surfaces: BTreeMap<usize, Vec<RegionSurface>>,
    /// `prepare_lightning_infill_data` — the per-object generator built on the
    /// expanded surfaces (`PrintObject.cpp:2711`). Shared between the
    /// `sparse_anchoring` anchor lines and the `fill_entities` emit.
    pub(in crate::project_slice) lightning_generator:
        Option<std::rc::Rc<crate::fill::lightning::generator::Generator>>,
}
