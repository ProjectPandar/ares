mod z_paths;

use std::mem;

use super::types::{OutPointId, OutRecId, PolyNodeContour, PolyNodeId, PolyNodeRecord};
use super::{ClipOperation, Clipper, ClipperError, ClipperOptions, FillRule, PathRole};
use crate::geometry::{ExPolygon, Polygon, Polyline};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct PolyTree {
    nodes: Vec<PolyNodeRecord>,
    children: Vec<PolyNodeId>,
}

impl PolyTree {
    pub(crate) fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            children: Vec::new(),
        }
    }

    fn expolygon_count(&self, node: PolyNodeId) -> usize {
        if matches!(self.nodes[node.0].contour, Some(PolyNodeContour::Open(_))) {
            return 0;
        }
        1 + self.nodes[node.0]
            .children
            .iter()
            .flat_map(|hole| self.nodes[hole.0].children.iter())
            .map(|&island| self.expolygon_count(island))
            .sum::<usize>()
    }

    pub(crate) fn into_expolygons(mut self) -> Vec<ExPolygon> {
        let capacity = self
            .children
            .iter()
            .map(|&root| self.expolygon_count(root))
            .sum();
        let roots = mem::take(&mut self.children);
        let mut expolygons = Vec::with_capacity(capacity);
        for root in roots {
            if matches!(self.nodes[root.0].contour, Some(PolyNodeContour::Closed(_))) {
                append_expolygon(&mut self.nodes, root, &mut expolygons);
            }
        }
        expolygons
    }

    /// `traverse_pt_outside_in` (`ClipperUtils.cpp:992-1010`): at each
    /// PolyNode level, chain-order the siblings by their first contour
    /// point (`chain_clipper_polynodes` → `chain_segments_greedy`,
    /// `ShortestPath.cpp:1998-2013`), emit each contour (holes reversed to
    /// CCW), then recurse into that node's children before the next
    /// sibling — producing the outside-in nesting order `union_pt_chained_
    /// outside_in` consumes.
    pub(crate) fn into_outside_in_contours(mut self) -> Vec<Vec<crate::geometry::Point>> {
        let mut out = Vec::new();
        let roots = mem::take(&mut self.children);
        self.outside_in(&roots, &mut out);
        out
    }

    fn outside_in(&mut self, siblings: &[PolyNodeId], out: &mut Vec<Vec<crate::geometry::Point>>) {
        // Order this sibling level by nearest-neighbor chaining on each
        // contour's first point (`chain_clipper_polynodes`).
        let order = crate::geometry::chain_points::chain_points(
            &siblings
                .iter()
                .filter_map(|&id| match &self.nodes[id.0].contour {
                    Some(PolyNodeContour::Closed(polygon)) => polygon.points().first().copied(),
                    _ => None,
                })
                .collect::<Vec<_>>(),
        );
        let closed: Vec<PolyNodeId> = siblings
            .iter()
            .copied()
            .filter(|&id| matches!(self.nodes[id.0].contour, Some(PolyNodeContour::Closed(_))))
            .collect();
        for &index in &order {
            let id = closed[index];
            let hole = self.is_hole(id);
            let mut polygon = match self.nodes[id.0].contour.take() {
                Some(PolyNodeContour::Closed(polygon)) => polygon,
                _ => unreachable!(),
            };
            // `if (node->IsHole()) retval->back().reverse()` — CCW.
            if hole {
                polygon.reverse();
            }
            out.push(polygon.into_points());
            let children = mem::take(&mut self.nodes[id.0].children);
            self.outside_in(&children, out);
        }
    }

    fn is_hole(&self, id: PolyNodeId) -> bool {
        let mut is_hole = false;
        let mut parent = self.nodes[id.0].parent;
        while let Some(node) = parent {
            is_hole = !is_hole;
            parent = self.nodes[node.0].parent;
        }
        is_hole
    }

    pub(crate) fn into_open_polylines(mut self) -> Vec<Polyline> {
        let roots = mem::take(&mut self.children);
        let mut polylines = Vec::new();
        for root in roots {
            if let Some(PolyNodeContour::Open(polyline)) = self.nodes[root.0].contour.take() {
                polylines.push(polyline);
            }
        }
        polylines
    }

    pub(crate) fn remove_outermost_polygon(&mut self) {
        if self.children.len() != 1 || self.nodes[self.children[0].0].children.is_empty() {
            self.nodes.clear();
            self.children.clear();
            return;
        }
        let outer = self.children[0];
        self.children = mem::take(&mut self.nodes[outer.0].children);
        self.nodes[outer.0].contour = None;
        for &root in &self.children {
            self.nodes[root.0].parent = None;
        }
    }
}

impl Clipper {
    pub(super) fn build_polytree(&mut self) -> PolyTree {
        let mut nodes = Vec::with_capacity(self.out_recs.len());
        let mut node_by_out_rec = vec![None; self.out_recs.len()];

        for (index, node_slot) in node_by_out_rec.iter_mut().enumerate() {
            let out_rec = OutRecId(index);
            let Some(points) = self.out_recs[index].points else {
                continue;
            };
            let output = self.output_kernel_points(points);
            let is_open = self.out_recs[index].is_open;
            if is_open && output.len() < 2 || !is_open && output.len() < 3 {
                continue;
            }
            if !is_open {
                self.normalize_tree_first_left(out_rec);
            }
            let z = self
                .z_intersections
                .as_ref()
                .map(|_| output.iter().map(|point| point.z).collect());
            let xy = output.iter().map(|point| point.xy).collect();
            let contour = if is_open {
                PolyNodeContour::Open(Polyline::new(xy))
            } else {
                PolyNodeContour::Closed(Polygon::new(xy))
            };
            let node = PolyNodeId(nodes.len());
            nodes.push(PolyNodeRecord {
                parent: None,
                children: Vec::new(),
                contour: Some(contour),
                z,
            });
            *node_slot = Some(node);
        }

        let mut children = Vec::with_capacity(nodes.len());
        for (index, &node) in node_by_out_rec.iter().enumerate() {
            let Some(node) = node else {
                continue;
            };
            let parent = if self.out_recs[index].is_open {
                None
            } else {
                self.out_recs[index]
                    .first_left
                    .and_then(|out_rec| node_by_out_rec[out_rec.0])
            };
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

    fn output_kernel_points(&self, start: OutPointId) -> Vec<super::z::KernelPoint> {
        let first = self.out_points.point(start).previous;
        let mut point = first;
        let mut points = Vec::new();
        loop {
            let output = self.out_points.point(point);
            points.push(output.point);
            point = output.previous;
            if point == first {
                return points;
            }
        }
    }
}

pub(crate) fn union_ex(
    polygons: &[Polygon],
    fill_rule: FillRule,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let mut paths_clipper = Clipper::new(ClipperOptions::default());
    paths_clipper.add_closed_paths(polygons, PathRole::Subject)?;
    let paths = paths_clipper.execute_paths(ClipOperation::Union, fill_rule, fill_rule)?;
    if paths.is_empty() {
        return Ok(Vec::new());
    }

    let mut tree_clipper = Clipper::new(ClipperOptions::default());
    assert!(
        tree_clipper
            .add_closed_paths(&paths, PathRole::Subject)
            .expect("first-pass output paths remain inside the validated Clipper range"),
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
    let contour = match nodes[node.0].contour.take() {
        Some(PolyNodeContour::Closed(polygon)) => polygon,
        Some(PolyNodeContour::Open(_)) => unreachable!("open records are PolyTree roots"),
        None => unreachable!("unconsumed contour node"),
    };
    let hole_nodes = mem::take(&mut nodes[node.0].children);
    let mut holes = Vec::with_capacity(hole_nodes.len());
    let mut nested_islands = Vec::with_capacity(hole_nodes.len());
    for hole in hole_nodes {
        let hole_polygon = match nodes[hole.0].contour.take() {
            Some(PolyNodeContour::Closed(polygon)) => polygon,
            _ => unreachable!("holes are closed contours"),
        };
        holes.push(hole_polygon);
        nested_islands.push(mem::take(&mut nodes[hole.0].children));
    }
    expolygons.push(ExPolygon::new(contour, holes));
    for islands in nested_islands {
        for island in islands {
            append_expolygon(nodes, island, expolygons);
        }
    }
}

/// `union_pt` + contour extraction (`ClipperUtils.cpp:955`): the union
/// executed with `pftEvenOdd`, contours returned in the PolyTree walk
/// order (which reorders each contour's points relative to the input).
/// `union_pt_chained_outside_in` (`ClipperUtils.cpp:1012-1016`): the
/// `pftEvenOdd` union's PolyTree walked outside-in with nearest-neighbor
/// sibling chaining, holes reversed to CCW. The flat `union_ex` +
/// `contour.chain(holes)` previously used dropped both the sibling
/// chaining and the >2-level nesting the upstream traversal preserves.
pub(crate) fn union_contours(
    polygons: &[Polygon],
) -> Result<Vec<Vec<crate::geometry::Point>>, ClipperError> {
    let mut clipper = Clipper::new(ClipperOptions::default());
    clipper.add_closed_paths(polygons, PathRole::Subject)?;
    Ok(clipper
        .execute_polytree(ClipOperation::Union, FillRule::EvenOdd, FillRule::EvenOdd)
        .into_outside_in_contours())
}
