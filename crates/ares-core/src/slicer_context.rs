mod rng;
pub(crate) use rng::Mt19937;

/// Execution-local state for one project slice.
pub struct SlicerContext {
    seed: u32,
    rng: Mt19937,
}

impl SlicerContext {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            rng: Mt19937::new(seed),
        }
    }

    pub(crate) fn seed(&self) -> u32 {
        self.seed
    }

    pub(crate) fn rng(&mut self) -> &mut Mt19937 {
        &mut self.rng
    }
}
