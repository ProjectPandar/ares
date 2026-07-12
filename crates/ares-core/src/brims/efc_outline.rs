use super::combine::Bounds;

pub(super) fn bounds_or_adjusted(bounds: Bounds, offset: Option<f64>) -> Option<Bounds> {
    let Some(offset) = offset else {
        return Some(bounds);
    };
    let adjusted = (
        bounds.0 + offset,
        bounds.1 + offset,
        bounds.2 - offset,
        bounds.3 - offset,
    );
    (adjusted.0 < adjusted.2 && adjusted.1 < adjusted.3).then_some(adjusted)
}
