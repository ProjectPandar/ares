mod rng;
pub(crate) use rng::Mt19937;
mod support_connection;
pub(crate) use support_connection::SupportConnectionCosts;

/// Execution-local state for one project slice.
pub struct SlicerContext {
    rng: Mt19937,
    support_connection_costs: Option<SupportConnectionCosts>,
}

impl SlicerContext {
    pub fn new(seed: u32) -> Self {
        Self {
            rng: Mt19937::new(seed),
            support_connection_costs: None,
        }
    }

    pub(crate) fn rng(&mut self) -> &mut Mt19937 {
        &mut self.rng
    }

    pub(crate) fn support_connection_costs(&mut self, line_spacing: f64) -> SupportConnectionCosts {
        *self
            .support_connection_costs
            .get_or_insert_with(|| SupportConnectionCosts::new(line_spacing))
    }
}
