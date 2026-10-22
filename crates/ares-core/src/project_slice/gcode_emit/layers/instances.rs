//! `PrintApply.cpp:129–167`, `ShortestPath.cpp:2015–2036`: printable instances.

use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;

#[derive(Clone, Copy)]
pub(in crate::project_slice::gcode_emit) struct Instance {
    pub(super) copy: usize,
    pub(in crate::project_slice::gcode_emit) label_id: u32,
    pub(in crate::project_slice::gcode_emit) shift: crate::geometry::Point,
    pub(super) position: usize,
}

pub(super) fn collect(traversal: &PreparedPostClassicTraversal) -> Vec<Vec<Instance>> {
    let mut instances = Vec::with_capacity(traversal.objects.len());
    let mut owners = Vec::new();
    let mut centers = Vec::new();
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
                centers.push((shift.x(), shift.y()));
                Instance {
                    copy,
                    label_id: object.instances()[source_instance]
                        .loaded_label_id()
                        .unwrap_or(0),
                    shift,
                    position: 0,
                }
            })
            .collect::<Vec<_>>();
        instances.push(copies);
    }
    // The existing chunk adapter reverses the source chain before dispatch.
    let order = if as_object_list(traversal) {
        super::object_order::model_order(&owners)
    } else {
        super::object_order::chain_instance_order(&centers, seed(traversal))
            .into_iter()
            .rev()
            .collect()
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
    traversal.resolved.views.full.process.print.print_order
        == crate::ProcessPrintOrder::AsObjectList
}

fn seed(traversal: &PreparedPostClassicTraversal) -> (i64, i64) {
    let print = &traversal.resolved.views.full.project.print;
    // GCode.cpp:5113 constructs Point directly, not Point::new_scale.
    (
        print.wipe_tower_x.0[0].0 as i64,
        print.wipe_tower_y.0[0].0 as i64,
    )
}

/// `GCode.cpp:5106–5138` chains only instances participating in this tool.
pub(super) fn order(
    mut entries: Vec<super::tool_buckets::Entry>,
    traversal: &PreparedPostClassicTraversal,
) -> Vec<super::tool_buckets::Entry> {
    entries.sort_by_key(|entry| (entry.source.object_index, entry.instance.copy));
    if as_object_list(traversal) {
        // `GCode.cpp:5129-5133`: AsObjectList feeds the global ordering into
        // `sort_print_object_instances`, so the per-tool chain is bypassed and
        // the bucket positions (model order) are already the emission order.
        entries.sort_by_key(|entry| entry.instance.position);
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
