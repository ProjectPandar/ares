// SeamContour.cpp::BoxEntry and Resolver::query. Query results are sorted by
// contour index, so the balanced box tree's partition order is not observable.
use super::Contour;

struct Entry {
    index: usize,
    min: [f64; 2],
    max: [f64; 2],
}

enum Kind {
    Leaf(usize),
    Branch([usize; 2]),
}

struct Node {
    min: [f64; 2],
    max: [f64; 2],
    kind: Kind,
}

pub(super) struct Index {
    nodes: Vec<Node>,
    root: Option<usize>,
}

impl Index {
    pub(super) fn new(contours: &[Contour]) -> Self {
        let mut entries = contours
            .iter()
            .enumerate()
            .filter(|(_, contour)| !contour.points.is_empty())
            .map(|(index, contour)| Entry {
                index,
                min: contour.min,
                max: contour.max,
            })
            .collect::<Vec<_>>();
        let mut tree = Self {
            nodes: Vec::with_capacity(entries.len() * 2),
            root: None,
        };
        if !entries.is_empty() {
            tree.root = Some(tree.build(&mut entries));
        }
        tree
    }

    fn build(&mut self, entries: &mut [Entry]) -> usize {
        let mut min = entries[0].min;
        let mut max = entries[0].max;
        for entry in &entries[1..] {
            for axis in 0..2 {
                min[axis] = min[axis].min(entry.min[axis]);
                max[axis] = max[axis].max(entry.max[axis]);
            }
        }
        let kind = if entries.len() == 1 {
            Kind::Leaf(entries[0].index)
        } else {
            let axis = usize::from(max[1] - min[1] > max[0] - min[0]);
            let middle = entries.len() / 2;
            entries.select_nth_unstable_by(middle, |a, b| {
                let a_center = (a.min[axis] + a.max[axis]) * 0.5;
                let b_center = (b.min[axis] + b.max[axis]) * 0.5;
                a_center.total_cmp(&b_center)
            });
            let (left, right) = entries.split_at_mut(middle);
            Kind::Branch([self.build(left), self.build(right)])
        };
        let index = self.nodes.len();
        self.nodes.push(Node { min, max, kind });
        index
    }

    pub(super) fn query(&self, point: [f64; 2], reach: f64) -> Vec<usize> {
        let min = [point[0] - reach, point[1] - reach];
        let max = [point[0] + reach, point[1] + reach];
        let mut result = Vec::new();
        if let Some(root) = self.root {
            self.visit(root, min, max, &mut result);
        }
        result.sort_unstable();
        result
    }

    fn visit(&self, index: usize, min: [f64; 2], max: [f64; 2], result: &mut Vec<usize>) {
        let node = &self.nodes[index];
        if (0..2).any(|axis| node.min[axis] > max[axis] || node.max[axis] < min[axis]) {
            return;
        }
        match node.kind {
            Kind::Leaf(contour) => result.push(contour),
            Kind::Branch(children) => {
                for child in children {
                    self.visit(child, min, max, result);
                }
            }
        }
    }
}
