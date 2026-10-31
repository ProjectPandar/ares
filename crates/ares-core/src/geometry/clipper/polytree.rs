// ClipperUtils.cpp:283–390,1011–1018: direct PolyTree output, not a second union.
mod outside_in;
mod stray_holes;

use super::{ClipOperation, ClipperError, FillRule, backend};
use crate::geometry::{CoordinateScale, ExPolygon, Point, Polygon, clipper2 as c2};

pub(crate) struct PolyTree {
    tree: c2::PolyTree64,
}

impl PolyTree {
    pub(super) fn new(tree: c2::PolyTree64) -> Self {
        Self { tree }
    }

    pub(crate) fn into_expolygons(self, scale: CoordinateScale) -> Vec<ExPolygon> {
        let capacity = self
            .tree
            .root()
            .children()
            .iter()
            .map(|&i| count(&self.tree, i))
            .sum();
        let mut output = Vec::with_capacity(capacity);
        let mut stray = Vec::new();
        for &root in self.tree.root().children() {
            append(&self.tree, root, &mut output, &mut stray);
        }
        stray_holes::attach(&self.tree, &mut output, &stray, scale);
        output
    }

    pub(crate) fn into_outside_in_contours(self, scale: CoordinateScale) -> Vec<Vec<Point>> {
        let mut output = Vec::new();
        outside_in::traverse(
            &self.tree,
            self.tree.root().children(),
            false,
            scale,
            &mut output,
        );
        output
    }
}

fn count(tree: &c2::PolyTree64, index: usize) -> usize {
    1 + tree.nodes[index]
        .children()
        .iter()
        .flat_map(|&hole| tree.nodes[hole].children())
        .map(|&island| count(tree, island))
        .sum::<usize>()
}

fn append(
    tree: &c2::PolyTree64,
    index: usize,
    output: &mut Vec<ExPolygon>,
    stray: &mut Vec<usize>,
) {
    let node = &tree.nodes[index];
    if c2::area(node.polygon()) < 0.0 {
        stray.push(index);
        for &child in node.children() {
            append(tree, child, output, stray);
        }
        return;
    }
    let out = output.len();
    output.push(ExPolygon::new(
        backend::polygon(node.polygon()),
        Vec::with_capacity(node.count()),
    ));
    for &child in node.children() {
        let child_node = &tree.nodes[child];
        if c2::area(child_node.polygon()) > 0.0 {
            append(tree, child, output, stray);
        } else {
            output[out]
                .holes_mut()
                .push(backend::polygon(child_node.polygon()));
            for &island in child_node.children() {
                append(tree, island, output, stray);
            }
        }
    }
}

pub(crate) fn union_ex(
    polygons: &[Polygon],
    fill: FillRule,
) -> Result<Vec<ExPolygon>, ClipperError> {
    Ok(backend::clip_tree(
        backend::paths(polygons),
        Vec::new(),
        ClipOperation::Union,
        fill,
    )?
    .into_expolygons(CoordinateScale::Normal))
}

pub(crate) fn union_contours(
    polygons: &[Polygon],
    scale: CoordinateScale,
) -> Result<Vec<Vec<Point>>, ClipperError> {
    Ok(backend::clip_tree(
        backend::paths(polygons),
        Vec::new(),
        ClipOperation::Union,
        FillRule::EvenOdd,
    )?
    .into_outside_in_contours(scale))
}
