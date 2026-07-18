use std::mem;

#[cfg(test)]
use std::slice;

use super::types::{OutPointId, OutRecId, PolyNodeId, PolyNodeRecord};
use super::{ClipOperation, ClipperError, ClipperOptions, ClosedClipper, FillRule, PathRole};
use crate::geometry::{ExPolygon, Polygon};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct PolyTree {
    nodes: Vec<PolyNodeRecord>,
    children: Vec<PolyNodeId>,
}

#[cfg(test)]
#[derive(Clone, Copy)]
pub(crate) struct PolyNode<'a> {
    tree: &'a PolyTree,
    id: PolyNodeId,
}

#[cfg(test)]
pub(crate) struct PolyNodeChildren<'a> {
    tree: &'a PolyTree,
    ids: slice::Iter<'a, PolyNodeId>,
}

impl PolyTree {
    pub(super) fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            children: Vec::new(),
        }
    }

    #[cfg(test)]
    pub(crate) fn total(&self) -> usize {
        self.nodes.len()
    }

    #[cfg(test)]
    pub(crate) fn children(&self) -> PolyNodeChildren<'_> {
        PolyNodeChildren {
            tree: self,
            ids: self.children.iter(),
        }
    }

    fn expolygon_count(&self, node: PolyNodeId) -> usize {
        1 + self.nodes[node.0]
            .children
            .iter()
            .flat_map(|hole| self.nodes[hole.0].children.iter())
            .map(|&island| self.expolygon_count(island))
            .sum::<usize>()
    }

    fn into_expolygons(mut self) -> Vec<ExPolygon> {
        let capacity = self
            .children
            .iter()
            .map(|&root| self.expolygon_count(root))
            .sum();
        let roots = mem::take(&mut self.children);
        let mut expolygons = Vec::with_capacity(capacity);
        for root in roots {
            append_expolygon(&mut self.nodes, root, &mut expolygons);
        }
        expolygons
    }
}

#[cfg(test)]
impl<'a> PolyNode<'a> {
    pub(crate) fn contour(self) -> &'a Polygon {
        self.tree.nodes[self.id.0]
            .contour
            .as_ref()
            .expect("live PolyTree node has a contour")
    }

    pub(crate) fn children(self) -> PolyNodeChildren<'a> {
        PolyNodeChildren {
            tree: self.tree,
            ids: self.tree.nodes[self.id.0].children.iter(),
        }
    }

    pub(crate) fn is_hole(self) -> bool {
        let mut is_hole = false;
        let mut parent = self.tree.nodes[self.id.0].parent;
        while let Some(node) = parent {
            is_hole = !is_hole;
            parent = self.tree.nodes[node.0].parent;
        }
        is_hole
    }
}

#[cfg(test)]
impl<'a> Iterator for PolyNodeChildren<'a> {
    type Item = PolyNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.ids.next().map(|&id| PolyNode {
            tree: self.tree,
            id,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.ids.size_hint()
    }
}

#[cfg(test)]
impl ExactSizeIterator for PolyNodeChildren<'_> {}

impl ClosedClipper {
    pub(super) fn build_polytree(&mut self) -> PolyTree {
        let mut nodes = Vec::with_capacity(self.out_recs.len());
        let mut node_by_out_rec = vec![None; self.out_recs.len()];

        for (index, node_slot) in node_by_out_rec.iter_mut().enumerate() {
            let out_rec = OutRecId(index);
            let Some(points) = self.out_recs[index].points else {
                continue;
            };
            let contour = self.output_polygon(points);
            if contour.points().len() < 3 {
                continue;
            }

            self.normalize_tree_first_left(out_rec);
            let node = PolyNodeId(nodes.len());
            nodes.push(PolyNodeRecord {
                parent: None,
                children: Vec::new(),
                contour: Some(contour),
            });
            *node_slot = Some(node);
        }

        let mut children = Vec::with_capacity(nodes.len());
        for (index, &node) in node_by_out_rec.iter().enumerate() {
            let Some(node) = node else {
                continue;
            };
            let parent = self.out_recs[index]
                .first_left
                .and_then(|out_rec| node_by_out_rec[out_rec.0]);
            if let Some(parent) = parent {
                nodes[node.0].parent = Some(parent);
                nodes[parent.0].children.push(node);
            } else {
                children.push(node);
            }
        }

        PolyTree { nodes, children }
    }

    fn normalize_tree_first_left(&mut self, out_rec: OutRecId) {
        let is_hole = self.out_recs[out_rec.0].is_hole;
        let mut first_left = self.out_recs[out_rec.0].first_left;
        while let Some(candidate) = first_left {
            let candidate_rec = self.out_recs[candidate.0];
            if candidate_rec.is_hole != is_hole && candidate_rec.points.is_some() {
                break;
            }
            first_left = candidate_rec.first_left;
        }
        self.out_recs[out_rec.0].first_left = first_left;
    }

    fn output_polygon(&self, start: OutPointId) -> Polygon {
        let first = self.out_points.point(start).previous;
        let mut point = first;
        let mut points = Vec::new();
        loop {
            let output = self.out_points.point(point);
            points.push(output.point);
            point = output.previous;
            if point == first {
                return Polygon::new(points);
            }
        }
    }
}

pub(crate) fn union_ex(
    polygons: &[Polygon],
    fill_rule: FillRule,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let mut paths_clipper = ClosedClipper::new(ClipperOptions::default());
    paths_clipper.add_closed_paths(polygons, PathRole::Subject)?;
    let paths = paths_clipper.execute_paths(ClipOperation::Union, fill_rule, fill_rule);
    if paths.is_empty() {
        return Ok(Vec::new());
    }

    let mut tree_clipper = ClosedClipper::new(ClipperOptions::default());
    assert!(
        tree_clipper
            .add_closed_paths(&paths, PathRole::Subject)
            .expect("first-pass output paths must remain inside the validated Clipper range"),
        "nonempty first-pass output must contain a valid closed path"
    );
    Ok(tree_clipper
        .execute_polytree(ClipOperation::Union, fill_rule, fill_rule)
        .into_expolygons())
}

fn append_expolygon(
    nodes: &mut [PolyNodeRecord],
    node: PolyNodeId,
    expolygons: &mut Vec<ExPolygon>,
) {
    let contour = nodes[node.0]
        .contour
        .take()
        .expect("unconsumed contour node");
    let hole_nodes = mem::take(&mut nodes[node.0].children);
    let mut holes = Vec::with_capacity(hole_nodes.len());
    let mut nested_islands = Vec::with_capacity(hole_nodes.len());
    for hole in hole_nodes {
        holes.push(nodes[hole.0].contour.take().expect("unconsumed hole node"));
        nested_islands.push(mem::take(&mut nodes[hole.0].children));
    }

    expolygons.push(ExPolygon::new(contour, holes));
    for islands in nested_islands {
        for island in islands {
            append_expolygon(nodes, island, expolygons);
        }
    }
}
