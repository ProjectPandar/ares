use super::exact_layer;

pub(super) fn compare(
    layer: usize,
    expected: &[Vec<String>],
    actual: &[Vec<String>],
) -> Result<(), String> {
    exact_layer(layer, "island lifecycle", &expected, &actual)
}
