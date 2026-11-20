use ares_core::Placement;

#[derive(clap::Args)]
pub(crate) struct DirectArrangeArgs {
    /// Preserve positions (0) or arrange separate STL inputs (1, the default).
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=1))]
    arrange: Option<u8>,
}

impl DirectArrangeArgs {
    pub(super) fn requested(&self) -> bool {
        self.arrange.is_some()
    }

    pub(super) fn placement(&self, input_count: usize) -> Placement {
        if self.arrange == Some(0) {
            Placement::None
        } else if input_count > 1 {
            Placement::Arrange
        } else {
            Placement::Center
        }
    }
}
