use super::{PreparedPostClassicTraversal, motion};
use crate::project_slice::gcode_emit::layers::schedule::LayerToPrint;

pub(in crate::project_slice::gcode_emit::layers) fn prepare(
    traversal: &PreparedPostClassicTraversal,
    entries: &[LayerToPrint],
    state: &mut motion::EmitState,
) -> Option<bool> {
    let entry = entries.first()?;
    let object = &traversal.objects[entry.object_index];
    let floor = object.belt_floor()?;
    let index = entry.object_layer?;
    let print_z = object.physical_layers()[index].print_z;
    let first = traversal
        .resolved
        .views
        .full
        .process
        .print
        .initial_layer_print_height
        .0;
    let bands = motion::BeltBands {
        floor,
        first,
        print_z,
    };
    state.belt_bands = Some(bands);

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
            floor.height(xy, print_z, 0.)
        })
        .reduce(f64::min)
        .map(|min| min >= bands.thickness() - 1e-4)
}
