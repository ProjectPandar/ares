use crate::geometry::{CoordinateScale, ExPolygon, Polygon};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RegionExpansionParameters {
    pub(crate) tiny_expansion: f32,
    pub(crate) initial_step: f32,
    pub(crate) other_step: f32,
    pub(crate) num_other_steps: usize,
    pub(crate) max_inflation: f32,
    pub(crate) arc_tolerance: f64,
    pub(crate) shortest_edge_length: f64,
}

impl RegionExpansionParameters {
    pub(crate) fn build(
        full_expansion: f32,
        expansion_step: f32,
        max_nr_expansion_steps: usize,
        scale: CoordinateScale,
    ) -> Self {
        assert!(full_expansion > 0.0);
        assert!(expansion_step > 0.0);
        assert!(max_nr_expansion_steps > 0);

        let mut tiny_expansion = (0.25_f32 * full_expansion).min(0.05_f32 / scale.factor() as f32);
        let mut nsteps = ((full_expansion - tiny_expansion) / expansion_step).ceil() as usize;
        nsteps = nsteps.min(max_nr_expansion_steps);
        assert!(nsteps > 0);
        let mut initial_step = (full_expansion - tiny_expansion) / nsteps as f32;
        if nsteps > 1 && 0.25_f64 * f64::from(initial_step) < f64::from(tiny_expansion) {
            nsteps = ((f64::from(full_expansion - tiny_expansion)
                / (4.0_f64 * f64::from(tiny_expansion)))
            .floor() as usize)
                .max(1);
            initial_step = (full_expansion - tiny_expansion) / nsteps as f32;
        }
        if 0.25_f64 * f64::from(initial_step) < f64::from(tiny_expansion) || nsteps == 1 {
            tiny_expansion = 0.2_f32 * full_expansion;
            initial_step = 0.8_f32 * full_expansion;
        }

        Self {
            tiny_expansion,
            initial_step,
            other_step: initial_step,
            num_other_steps: nsteps - 1,
            max_inflation: (f64::from(tiny_expansion + nsteps as f32 * initial_step) * 1.1_f64)
                as f32,
            arc_tolerance: 0.1_f64 / scale.factor(),
            shortest_edge_length: f64::from(initial_step) * 0.005_f64,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WaveSeed {
    pub(crate) src: u32,
    pub(crate) boundary: u32,
    pub(crate) path: Polygon,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RegionExpansion {
    pub(crate) polygon: Polygon,
    pub(crate) src_id: u32,
    pub(crate) boundary_id: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RegionExpansionEx {
    pub(crate) expolygon: ExPolygon,
    pub(crate) src_id: u32,
    pub(crate) boundary_id: u32,
}
