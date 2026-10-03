//! Flattened down-links from `TreeSupport3D.cpp::organic_draw_branches`.

use super::super::support_element::SupportElement;
use std::{collections::VecDeque, mem};

#[derive(Clone, Copy)]
pub(super) struct ElementLink {
    pub(super) layer: usize,
    pub(super) index: usize,
    pub(super) below: Option<usize>,
}

pub(in super::super) struct BranchLinks {
    pub(super) elements: Vec<ElementLink>,
    pub(super) layers: Vec<usize>,
}

impl BranchLinks {
    pub(super) fn new(graph: &[VecDeque<SupportElement>]) -> Self {
        let mut elements = Vec::with_capacity(graph.iter().map(VecDeque::len).sum());
        let mut layers = Vec::with_capacity(graph.len() + 1);
        let mut incoming: Vec<Option<usize>> = Vec::new();
        let mut next = Vec::new();
        layers.push(0);
        for (layer, nodes) in graph.iter().enumerate() {
            next.clear();
            next.resize(graph.get(layer + 1).map_or(0, VecDeque::len), None);
            for (index, node) in nodes.iter().enumerate() {
                let below = incoming.get(index).copied().flatten();
                Self::map_parents(graph, layer, index, node, &mut next);
                elements.push(ElementLink {
                    layer,
                    index,
                    below,
                });
            }
            mem::swap(&mut incoming, &mut next);
            layers.push(elements.len());
        }
        Self { elements, layers }
    }

    fn map_parents(
        graph: &[VecDeque<SupportElement>],
        layer: usize,
        index: usize,
        node: &SupportElement,
        next: &mut [Option<usize>],
    ) {
        for &parent in &node.parents {
            if graph[layer + 1][parent as usize]
                .state
                .result_on_layer
                .is_some()
            {
                debug_assert!(next[parent as usize].is_none());
                next[parent as usize] = Some(index);
            }
        }
    }
}
