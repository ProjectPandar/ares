use super::super::hierarchy::PerimeterGeneratorLoop;
use super::types::{
    LowerFlowRoute, PendingExtrusionRole, PendingLoopRole, RouteFlows, SeedFrame, TraversalSeed,
};
use crate::project_slice::perimeters::types::Flow;

pub(super) fn classify_roots(
    roots: &[PerimeterGeneratorLoop],
    flows: RouteFlows,
) -> Vec<TraversalSeed> {
    roots
        .iter()
        .map(|root| classify_tree(root, &flows))
        .collect()
}

fn classify_tree(root: &PerimeterGeneratorLoop, flows: &RouteFlows) -> TraversalSeed {
    let mut frames = vec![SeedFrame {
        source: root,
        next_child: 0,
        children: Vec::with_capacity(root.children.len()),
    }];
    loop {
        let frame = frames.last_mut().expect("classification has a root frame");
        if let Some(child) = frame.source.children.get(frame.next_child) {
            frame.next_child += 1;
            frames.push(SeedFrame {
                source: child,
                next_child: 0,
                children: Vec::with_capacity(child.children.len()),
            });
            continue;
        }
        let frame = frames.pop().expect("completed frame exists");
        let seed = classify_node(frame.source, frame.children, flows);
        if let Some(parent) = frames.last_mut() {
            parent.children.push(seed);
        } else {
            return seed;
        }
    }
}

fn classify_node(
    source: &PerimeterGeneratorLoop,
    children: Vec<TraversalSeed>,
    flows: &RouteFlows,
) -> TraversalSeed {
    let extrusion_role = if source.depth == 0 {
        PendingExtrusionRole::ExternalPerimeter
    } else {
        PendingExtrusionRole::Perimeter
    };
    let loop_role = if !source.is_contour {
        PendingLoopRole::Hole
    } else if source.children.iter().all(|child| !child.is_contour) {
        PendingLoopRole::Internal
    } else {
        PendingLoopRole::Default
    };
    let route = match (extrusion_role, source.is_smaller_width_perimeter) {
        (PendingExtrusionRole::ExternalPerimeter, true) => LowerFlowRoute::SmallerExternal,
        (PendingExtrusionRole::ExternalPerimeter, false) => LowerFlowRoute::External,
        (PendingExtrusionRole::Perimeter, _) => LowerFlowRoute::Internal,
    };
    let flow = selected_flow(route, flows);
    TraversalSeed {
        polygon: source.polygon.clone(),
        depth: source.depth,
        is_contour: source.is_contour,
        is_smaller_width_perimeter: source.is_smaller_width_perimeter,
        extrusion_role,
        loop_role,
        route,
        width: flow.width,
        mm3_per_mm: flow.mm3_per_mm,
        children,
    }
}

fn selected_flow(route: LowerFlowRoute, flows: &RouteFlows) -> Flow {
    match route {
        LowerFlowRoute::SmallerExternal => flows.smaller_external,
        LowerFlowRoute::External => flows.external,
        LowerFlowRoute::Internal => flows.perimeter,
    }
}

#[cfg(test)]
mod tests;
