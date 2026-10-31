// ClipperUtils.cpp:1220–1287: reattach misnested loops before chained traversal.
use super::{backend, c2, stray_holes};
use crate::geometry::{CoordinateScale, Point, chain_points, fixed_gcc_sort_by};

struct Loop {
    node: usize,
    area: f64,
    next: Vec<usize>,
    moved: bool,
}

pub(super) fn traverse(
    tree: &c2::PolyTree64,
    nodes: &[usize],
    holes: bool,
    scale: CoordinateScale,
    output: &mut Vec<Vec<Point>>,
) {
    let mut fitting = Vec::new();
    let mut misfits = Vec::new();
    let mut loops: Vec<_> = nodes
        .iter()
        .enumerate()
        .map(|(i, &node)| {
            let path = &tree.nodes[node];
            let area = c2::area(path.polygon());
            if (area < 0.0) == holes {
                fitting.push(i);
            } else {
                misfits.push(i);
            }
            Loop {
                node,
                area,
                next: path.children().to_vec(),
                moved: false,
            }
        })
        .collect();
    if !misfits.is_empty() {
        fixed_gcc_sort_by(&mut fitting, |&a, &b| {
            loops[a].area.abs() < loops[b].area.abs()
        });
        let boxes: Vec<_> = nodes
            .iter()
            .map(|&node| c2::get_bounds_path(tree.nodes[node].polygon()))
            .collect();
        for misfit in misfits {
            attach_misfit(
                tree,
                &mut loops,
                &boxes,
                &fitting,
                (misfit, scale.scaled_delta(0.0001)),
            );
        }
    }
    let staying: Vec<_> = loops
        .iter()
        .enumerate()
        .filter(|(_, item)| !item.moved)
        .map(|(i, _)| i)
        .collect();
    let points: Vec<_> = staying
        .iter()
        .map(|&i| {
            let point = tree.nodes[loops[i].node].polygon()[0];
            Point::new(point.x, point.y)
        })
        .collect();
    for index in chain_points(&points) {
        let item = &loops[staying[index]];
        let mut polygon = backend::polygon(tree.nodes[item.node].polygon());
        if item.area < 0.0 {
            polygon.reverse();
        }
        output.push(polygon.into_points());
        traverse(tree, &item.next, item.area > 0.0, scale, output);
    }
}

// ClipperUtils.cpp:1244–1268: one misplaced loop, same candidate order.
fn attach_misfit(
    tree: &c2::PolyTree64,
    loops: &mut [Loop],
    boxes: &[c2::Rect64],
    fitting: &[usize],
    (misfit, epsilon): (usize, f64),
) {
    let bounds = boxes[misfit];
    let inner = backend::polygon(tree.nodes[loops[misfit].node].polygon());
    for &candidate in fitting {
        let outer = boxes[candidate];
        if loops[candidate].area.abs() <= loops[misfit].area.abs()
            || outer.left as f64 - epsilon > bounds.left as f64
            || outer.right as f64 + epsilon < bounds.right as f64
            || outer.top as f64 - epsilon > bounds.top as f64
            || outer.bottom as f64 + epsilon < bounds.bottom as f64
        {
            continue;
        }
        if stray_holes::encloses(
            &backend::polygon(tree.nodes[loops[candidate].node].polygon()),
            &inner,
        ) {
            let node = loops[misfit].node;
            loops[candidate].next.push(node);
            loops[misfit].moved = true;
            return;
        }
    }
}
