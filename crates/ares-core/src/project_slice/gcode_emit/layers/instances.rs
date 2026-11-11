//! `PrintApply.cpp:129–167`, `ShortestPath.cpp:2015–2036`: printable instances.

use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;

#[derive(Clone, Copy)]
pub(in crate::project_slice::gcode_emit) struct Instance {
    pub(super) copy: usize,
    pub(in crate::project_slice::gcode_emit) label_id: u32,
    pub(in crate::project_slice::gcode_emit) shift: crate::geometry::Point,
    pub(super) position: usize,
    pub(super) arrange_order: usize,
}

pub(super) fn collect(traversal: &PreparedPostClassicTraversal) -> Vec<Vec<Instance>> {
    let mut instances = Vec::with_capacity(traversal.objects.len());
    let mut owners = Vec::new();
    let mut centers = Vec::new();
    let mut model_indices = Vec::new();
    for (object_index, prepared) in traversal.objects.iter().enumerate() {
        let (source_index, transform_index) = prepared
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object
            .identity();
        let resolved = traversal
            .resolved
            .objects
            .iter()
            .find(|object| object.source_object_index == source_index)
            .expect("print object retains resolved instance membership");
        let object = &traversal.project.objects()[source_index];
        let center = crate::project_slice::centering::center(
            object,
            &resolved.print_objects[transform_index],
            traversal.scale,
        )
        .expect("sliced object retains its raw center");
        let copies = resolved.print_objects[transform_index]
            .instance_indices
            .iter()
            .enumerate()
            .map(|(copy, &source_instance)| {
                let shift = super::super::footprint::instance_shift(
                    &object.instances()[source_instance],
                    center,
                    traversal.scale,
                );
                owners.push((object_index, copy));
                model_indices.push(source_index);
                centers.push((shift.x(), shift.y()));
                Instance {
                    copy,
                    label_id: object.instances()[source_instance].labeled_id(),
                    shift,
                    position: 0,
                    arrange_order: 0,
                }
            })
            .collect::<Vec<_>>();
        instances.push(copies);
    }
    // Print.cpp:934 initializes by model-object index, not split PrintObject index.
    if as_object_list(traversal) {
        for (rank, index) in super::object_order::model_order(&model_indices)
            .into_iter()
            .enumerate()
        {
            let (object, copy) = owners[index];
            instances[object][copy].arrange_order = rank + 1;
        }
    }
    // Bucket identity remains independent of the model-order field.
    let print = &traversal.resolved.views.full.process.print;
    let order = if print.print_sequence == crate::ProcessPrintSequence::ByLayer
        && print.print_order == crate::ProcessPrintOrder::Default
    {
        super::object_order::reference_order(&centers, traversal.scale)
    } else {
        let mut order = super::object_order::chain_instance_order(&centers, seed(traversal));
        order.reverse();
        order
    };
    for (position, index) in order.into_iter().enumerate() {
        let (object, copy) = owners[index];
        instances[object][copy].position = position;
    }
    instances
}

/// `GCode.cpp:2791-2796`: `print_order == PrintOrder::AsObjectList` selects the
/// model order instead of the nearest-neighbour chain.
pub(super) fn as_object_list(traversal: &PreparedPostClassicTraversal) -> bool {
    let print = &traversal.resolved.views.full.process.print;
    print.print_sequence == crate::ProcessPrintSequence::ByLayer
        && print.print_order == crate::ProcessPrintOrder::AsObjectList
}

fn seed(traversal: &PreparedPostClassicTraversal) -> (i64, i64) {
    // GCode.cpp:7270 (v2.5.0-beta) constructs Point directly, not Point::new_scale.
    let [x, y] = super::super::wipe_tower::plate_position(traversal);
    (x as i64, y as i64)
}

/// `GCode.cpp:5106–5138` chains only instances participating in this tool.
pub(super) fn order(
    mut entries: Vec<super::tool_buckets::Entry>,
    traversal: &PreparedPostClassicTraversal,
) -> Vec<super::tool_buckets::Entry> {
    entries.sort_by_key(|entry| (entry.source.object_index, entry.instance.copy));
    if as_object_list(traversal) {
        // `GCode.cpp:5129-5133` uses the initialized global model order,
        // independently of the bucket's position key.
        entries.sort_by_key(|entry| entry.instance.arrange_order);
        return entries;
    }
    let centers = entries
        .iter()
        .map(|entry| {
            let point = entry.instance.shift;
            (point.x(), point.y())
        })
        .collect::<Vec<_>>();
    let order = super::object_order::chain_instance_order(&centers, seed(traversal));
    let mut entries = entries.into_iter().map(Some).collect::<Vec<_>>();
    order
        .into_iter()
        .rev()
        .map(|index| {
            entries[index]
                .take()
                .expect("instance permutation visits each entry once")
        })
        .collect()
}
