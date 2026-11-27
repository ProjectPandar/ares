use super::{EPSILON, LayerToPrint};

pub(super) fn pair_layers(
    destination: &mut Vec<(f64, LayerToPrint)>,
    object_index: usize,
    object_z: &[f64],
    support_z: &[f64],
) {
    let (mut object, mut support) = (0, 0);
    while object < object_z.len() || support < support_z.len() {
        let minimum = object_z
            .get(object)
            .copied()
            .unwrap_or(f64::MAX)
            .min(support_z.get(support).copied().unwrap_or(f64::MAX));
        let object_layer = object_z
            .get(object)
            .filter(|&&z| z <= minimum + EPSILON)
            .map(|_| object);
        let support_layer = support_z
            .get(support)
            .filter(|&&z| z <= minimum + EPSILON)
            .map(|_| support);
        let z = match (object_layer, support_layer) {
            (Some(object), Some(support)) => 0.5 * (object_z[object] + support_z[support]),
            (Some(object), None) => object_z[object],
            (None, Some(support)) => support_z[support],
            (None, None) => unreachable!("at least one next layer is at the minimum Z"),
        };
        destination.push((
            z,
            LayerToPrint {
                object_index,
                object_layer,
                support_layer,
                raft_layer: None,
                belt_prologue: None,
            },
        ));
        object += usize::from(object_layer.is_some());
        support += usize::from(support_layer.is_some());
    }
}
