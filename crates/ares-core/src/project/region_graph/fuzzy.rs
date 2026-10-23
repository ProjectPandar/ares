//! PrintApply.cpp1081–1110: intern fuzzy configurations in the shared PrintRegion table.
use super::{PaintedRegion, RegionGraph, register_region};
use crate::{ProcessFuzzySkinType, ProjectObject, geometry::fixed_gcc_sort_by};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FuzzyRegion {
    pub(crate) parent_region_id: usize,
    pub(crate) region_id: usize,
}

pub(crate) struct PaintedRegions {
    pub(crate) mmu: Vec<Vec<PaintedRegion>>,
    pub(crate) fuzzy: Vec<Vec<FuzzyRegion>>,
}

pub(super) fn register(source: &ProjectObject, graph: &mut RegionGraph) {
    if !source.is_fuzzy_skin_painted() {
        return;
    }
    for ((volumes, mmu), fuzzy) in graph
        .ranges
        .iter()
        .zip(&graph.painted_regions)
        .zip(&mut graph.fuzzy_regions)
    {
        for parent_region_id in volumes
            .iter()
            .filter_map(|volume| volume.region_id)
            .chain(mmu.iter().map(|region| region.region_id))
        {
            let mut configuration = graph.all_regions[parent_region_id].clone();
            if configuration.fuzzy_skin != ProcessFuzzySkinType::Disabled {
                configuration.fuzzy_skin = ProcessFuzzySkinType::All;
            }
            let region_id = register_region(&mut graph.all_regions, configuration);
            fuzzy.push(FuzzyRegion {
                parent_region_id,
                region_id,
            });
        }
        fixed_gcc_sort_by(fuzzy, |a, b| a.parent_region_id < b.parent_region_id);
    }
}
