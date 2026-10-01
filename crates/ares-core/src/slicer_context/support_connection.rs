//! `FillBase.cpp:2464-2466` first-call static costs, scoped to one fresh-process job.

#[derive(Clone, Copy)]
pub(crate) struct SupportConnectionCosts {
    pub(crate) low: f64,
    pub(crate) high: f64,
    pub(crate) very_high: f64,
}

impl SupportConnectionCosts {
    pub(super) fn new(line_spacing: f64) -> Self {
        Self {
            low: line_spacing * 1.3,
            high: line_spacing * 2.0,
            very_high: line_spacing * 3.0,
        }
    }
}
