//! `GCode.cpp:7077–7083,6421–6433`: support-role filament attribution and candidates.

use super::*;

pub(super) fn tools(
    traversal: &PreparedPostClassicTraversal,
    object: usize,
    support: Option<&InstalledSupportLayer>,
    raft: Option<&OrderedExtrusionLayer>,
    tools: &LayerTools,
) -> Vec<(usize, ExtrusionRole)> {
    let options = super::super::super::super::tool_ordering::object_options(traversal, object);
    if let Some(layer) = support {
        return super::super::super::tool_buckets::support::selectors(
            layer.support_fills.iter().map(|entity| entity.role()),
            options,
            traversal,
            &tools.extruders,
            [
                tools.wiping.support_tool(object, false),
                tools.wiping.support_tool(object, true),
            ],
        );
    }
    let Some(layer) = raft else {
        return Vec::new();
    };
    let mut role = ExtrusionRole::None;
    raft_paths(layer, &mut |_, next, _, _| {
        role = if role == ExtrusionRole::None || role == next {
            next
        } else {
            ExtrusionRole::Mixed
        };
    });
    super::super::super::tool_buckets::support::selectors(
        std::iter::once(role),
        options,
        traversal,
        &tools.extruders,
        [None; 2],
    )
}

pub(super) fn collect(
    search: &mut Search<'_>,
    (support, raft): (
        Option<&InstalledSupportLayer>,
        Option<&OrderedExtrusionLayer>,
    ),
    tools: &[(usize, ExtrusionRole)],
    (object, copy, shift): Placement,
) {
    let mut collect = |polyline: &Polyline3, role: ExtrusionRole, collection, leaf| {
        if !matches!(
            role,
            ExtrusionRole::SupportMaterial
                | ExtrusionRole::SupportMaterialInterface
                | ExtrusionRole::SupportTransition
        ) {
            return;
        }
        let &(filament, _) = tools
            .iter()
            .find(|(_, selected)| {
                *selected == ExtrusionRole::Mixed
                    || *selected == role
                    || (*selected == ExtrusionRole::SupportMaterial
                        && role == ExtrusionRole::SupportTransition)
            })
            .expect("candidate support role has a filament");
        let physical = search.physical(filament);
        search.fallback.collect(
            polyline,
            shift,
            physical,
            (object, copy, usize::MAX, collection, leaf),
        );
    };
    if let Some(layer) = support {
        let mut leaf = 0;
        for entity in &layer.support_fills {
            entity.for_each_path(&mut |path| {
                collect(&path.polyline, path.role, 0, leaf);
                leaf += 1;
            });
        }
    }
    if let Some(layer) = raft {
        raft_paths(layer, &mut collect);
    }
}

fn raft_paths(
    layer: &OrderedExtrusionLayer,
    visit: &mut impl FnMut(&Polyline3, ExtrusionRole, usize, usize),
) {
    for island in &layer.islands {
        let mut previous = None;
        let mut leaf = 0;
        for (entity, &collection) in island.entities.iter().zip(&island.infill_collections) {
            if previous != Some(collection) {
                previous = Some(collection);
                leaf = 0;
            }
            visit_fill(entity, &mut |polyline, role| {
                visit(polyline, role, collection, leaf);
                leaf += 1;
            });
        }
    }
}
