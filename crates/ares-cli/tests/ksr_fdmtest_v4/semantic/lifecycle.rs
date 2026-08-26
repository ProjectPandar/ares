use super::{exact_layer, model::LifecycleEvent};

pub(super) fn compare(
    layer: usize,
    expected: &[Vec<LifecycleEvent>],
    actual: &[Vec<LifecycleEvent>],
) -> Result<(), String> {
    exact_layer(layer, "island lifecycle", &expected, &actual)
}
