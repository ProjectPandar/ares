//! `TreeSupport3D.cpp::remove_deleted_elements`.

use std::{collections::VecDeque, mem};

use super::super::support_element::SupportElement;

pub(super) fn remove_deleted_elements(move_bounds: &mut [VecDeque<SupportElement>]) {
    let mut map_parents = Vec::<i32>::new();
    let mut map_current = Vec::<i32>::new();
    for layer in move_bounds.iter_mut().rev() {
        map_current.clear();
        compact_layer(layer, &map_parents, &mut map_current);
        mem::swap(&mut map_current, &mut map_parents);
    }
}

fn compact_layer(
    layer: &mut VecDeque<SupportElement>,
    map_parents: &[i32],
    map_current: &mut Vec<i32>,
) {
    let mut index = 0;
    while index < layer.len() {
        if !layer[index].state.bits.deleted {
            if map_parents.is_empty() {
                index += 1;
                continue;
            }
            for parent in &mut layer[index].parents {
                *parent = map_parents[*parent as usize];
            }
            index += 1;
            continue;
        }
        if map_current.is_empty() {
            map_current.extend((0..layer.len()).map(|index| index as i32));
        }
        while index < layer.len()
            && layer
                .back()
                .expect("current layer contains a node")
                .state
                .bits
                .deleted
        {
            layer.pop_back();
            map_current[layer.len()] = -1;
        }
        if index + 1 < layer.len() {
            layer[index] = layer
                .pop_back()
                .expect("live tail follows the deleted node");
            map_current[index] = -1;
            map_current[layer.len()] = index as i32;
        }
    }
}
