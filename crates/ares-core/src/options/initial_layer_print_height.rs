use crate::{SliceError, SliceOptions};

use super::defaults::DEFAULT_LAYER_HEIGHT;

const KEY: &str = "initial_layer_print_height";

impl SliceOptions {
    pub fn initial_layer_print_height(&self) -> Result<f64, SliceError> {
        super::parsing::parse_positive_number_or_string(
            KEY,
            self.values().get(KEY),
            DEFAULT_LAYER_HEIGHT,
        )
    }
}
