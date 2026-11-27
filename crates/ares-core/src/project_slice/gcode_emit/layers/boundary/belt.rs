use super::{PreparedPostClassicTraversal, motion};
use crate::project_slice::gcode_emit::layers::schedule::LayerToPrint;

fn bands(
    traversal: &PreparedPostClassicTraversal,
    entry: LayerToPrint,
) -> Option<motion::BeltBands> {
    let object = &traversal.objects[entry.object_index];
    let floor = object.belt_floor()?;
    let print_z = object.physical_layers()[entry.object_layer?].print_z;
    let first = traversal
        .resolved
        .views
        .full
        .process
        .print
        .initial_layer_print_height
        .0;
    Some(motion::BeltBands {
        floor,
        first,
        print_z,
    })
}

pub(in crate::project_slice::gcode_emit::layers) fn select_owner(
    traversal: &PreparedPostClassicTraversal,
    entry: LayerToPrint,
    state: &mut motion::EmitState,
) {
    state.belt_bands = bands(traversal, entry);
}

pub(in crate::project_slice::gcode_emit::layers) fn prepare(
    traversal: &PreparedPostClassicTraversal,
    entries: &[LayerToPrint],
    state: &mut motion::EmitState,
) -> Option<bool> {
    // GCode.cpp:6611–6644 chooses the first physical layer in PrintObject order.
    let entry = *entries
        .iter()
        .filter(|entry| entry.object_layer.is_some())
        .min_by_key(|entry| entry.object_index)?;
    let bands = bands(traversal, entry)?;
    state.belt_bands = Some(bands);
    let object = &traversal.objects[entry.object_index];
    let index = entry.object_layer?;

    // GCode::belt_layer_past_first_layer_band: a linear plane attains the
    // same minimum over the contour as over its axis-aligned bounding box.
    object
        .slices(index)?
        .iter()
        .flat_map(|slice| slice.contour().points())
        .map(|point| {
            let xy = [
                traversal.scale.unscale(point.x()),
                traversal.scale.unscale(point.y()),
            ];
            bands.floor.height(xy, bands.print_z, 0.)
        })
        .reduce(f64::min)
        .map(|min| min >= bands.thickness() - 1e-4)
}
