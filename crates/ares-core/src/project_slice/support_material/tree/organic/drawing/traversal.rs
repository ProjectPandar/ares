//! Source `TreeVisitor::visit_recursive`, with indexed rather than aliased paths.

use std::collections::VecDeque;

use super::super::support_element::SupportElement;

#[derive(Clone, Copy)]
pub(super) struct Node {
    pub(super) layer: usize,
    pub(super) index: usize,
}

pub(super) struct Branch {
    pub(super) path: Vec<Node>,
    pub(super) has_root: bool,
    pub(super) has_tip: bool,
}

pub(super) fn collect(graph: &mut [VecDeque<SupportElement>]) -> Vec<Vec<Branch>> {
    for layer in graph.iter_mut() {
        for element in layer {
            element.state.bits.marked = false;
        }
    }
    let mut trees = Vec::new();
    for upper in 1..graph.len() {
        let layer = upper - 1;
        for index in 0..graph[layer].len() {
            let element = &graph[layer][index];
            if !element.state.bits.marked && !element.parents.is_empty() {
                let mut branches = Vec::new();
                visit(graph, Node { layer, index }, &mut branches);
                trees.push(branches);
            }
        }
    }
    trees
}

fn visit(graph: &mut [VecDeque<SupportElement>], start: Node, branches: &mut Vec<Branch>) {
    let start_element = &mut graph[start.layer][start.index];
    debug_assert!(!start_element.state.bits.marked && !start_element.parents.is_empty());
    start_element.state.bits.marked = true;
    // All outgoing branches of this start node share the source root flag.
    let has_root = branches.is_empty();
    for parent in 0..graph[start.layer][start.index].parents.len() {
        let mut node = Node {
            layer: start.layer + 1,
            index: graph[start.layer][start.index].parents[parent] as usize,
        };
        let mut path = vec![start, node];
        let next_branch = loop {
            let element = &mut graph[node.layer][node.index];
            debug_assert!(!element.state.bits.marked);
            if element.parents.len() > 1 {
                break Some(node);
            }
            element.state.bits.marked = true;
            let Some(&parent) = element.parents.first() else {
                break None;
            };
            node = Node {
                layer: node.layer + 1,
                index: parent as usize,
            };
            path.push(node);
        };
        branches.push(Branch {
            path,
            has_root,
            has_tip: next_branch.is_none(),
        });
        if let Some(node) = next_branch {
            visit(graph, node, branches);
        }
    }
}
