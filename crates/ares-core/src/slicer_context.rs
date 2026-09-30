mod rng;
pub(crate) use rng::Mt19937;

/// Execution-local random state for one project slice.
pub struct SlicerContext {
    rng: Mt19937,
}

impl SlicerContext {
    pub fn new(seed: u32) -> Self {
        Self {
            rng: Mt19937::new(seed),
        }
    }

    pub(crate) fn rng(&mut self) -> &mut Mt19937 {
        &mut self.rng
    }
}
