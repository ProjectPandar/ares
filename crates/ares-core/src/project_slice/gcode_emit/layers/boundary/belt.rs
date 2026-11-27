use super::{PreparedPostClassicTraversal, motion};
use crate::project_slice::{
    gcode_emit::layers::schedule::LayerToPrint, support_material::InstalledSupportLayer,
};

pub(in crate::project_slice::gcode_emit::layers) fn prepare(
    traversal: &PreparedPostClassicTraversal,
    entries: &[LayerToPrint],
    support: &[Vec<InstalledSupportLayer>],
    state: &mut motion::EmitState,
) -> Option<bool> {
    let entry = entries.first()?;
    let object = &traversal.objects[entry.object_index];
    let object_z = entry
        .object_layer
        .map(|index| object.physical_layers()[index].print_z);
    let print_z = object_z.or_else(|| {
        entry
            .support_layer
            .map(|index| support[entry.object_index][index].print_z)
    });
    state.belt_bands = print_z.and_then(|z| bands(traversal, entry.object_index, z));
    let bands = state.belt_bands?;
    let index = entry.object_layer?;
    let print_z = object_z?;

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
            bands.floor.height(xy, print_z, 0.)
        })
        .reduce(f64::min)
        .map(|min| min >= bands.thickness() - 1e-4)
}

pub(in crate::project_slice::gcode_emit::layers) fn bands(
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    print_z: f64,
) -> Option<motion::BeltBands> {
    Some(motion::BeltBands {
        floor: traversal.objects[object_index].belt_floor()?,
        first: traversal
            .resolved
            .views
            .full
            .process
            .print
            .initial_layer_print_height
            .0,
        print_z,
    })
}
