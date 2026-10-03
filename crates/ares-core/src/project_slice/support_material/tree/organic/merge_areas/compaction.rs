//! `TreeSupport3D.cpp::merge_influence_areas_leaves/two_sets`.

use std::{mem, ops::Range};

use super::super::super::model_volumes::TreeModelVolumes;
use super::super::{TreeSettings, layer_growth::SupportElementMerging, pair_merge::merge_two};
use crate::geometry::{ClipperError, CoordinateScale};

pub(super) fn merge_leaves(
    volumes: &mut TreeModelVolumes<'_>,
    config: &TreeSettings,
    location: (usize, CoordinateScale),
    nodes: &mut [SupportElementMerging],
    range: Range<usize>,
) -> Result<usize, ClipperError> {
    let (layer, scale) = location;
    let mut end = range.end;
    let mut i = range.start;
    while i + 1 < end {
        let mut merged = false;
        let mut j = i + 1;
        while j != end {
            let (before, after) = nodes.split_at_mut(j);
            if !merge_two(
                volumes,
                config,
                layer,
                (&mut before[i], &mut after[0]),
                scale,
            )? {
                j += 1;
                continue;
            }
            end -= 1;
            if j != end {
                move_node(nodes, j, end);
            }
            merged = true;
            break;
        }
        if !merged {
            i += 1;
        }
    }
    Ok(end)
}

pub(super) fn merge_sets(
    volumes: &mut TreeModelVolumes<'_>,
    config: &TreeSettings,
    location: (usize, CoordinateScale),
    nodes: &mut [SupportElementMerging],
    ranges: (Range<usize>, Range<usize>),
) -> Result<usize, ClipperError> {
    let (layer, scale) = location;
    let (dst_range, src_range) = ranges;
    let mut dst_end = dst_range.end;
    let mut src_begin = src_range.start;
    for src in src_range.clone() {
        let mut dst = dst_range.start;
        let mut merged = None;
        while dst != dst_end {
            let (before, after) = nodes.split_at_mut(src);
            if !merge_two(
                volumes,
                config,
                layer,
                (&mut before[dst], &mut after[0]),
                scale,
            )? {
                dst += 1;
                continue;
            }
            merged = Some(dst);
            dst += 1;
            if src != src_begin {
                move_node(nodes, src, src_begin);
            }
            src_begin += 1;
            break;
        }
        while dst != dst_end {
            let (before, after) = nodes.split_at_mut(dst);
            if !merge_two(
                volumes,
                config,
                layer,
                (
                    &mut before[merged.expect("remaining destinations follow a merge")],
                    &mut after[0],
                ),
                scale,
            )? {
                dst += 1;
                continue;
            }
            dst_end -= 1;
            if dst != dst_end {
                move_node(nodes, dst, dst_end);
            }
        }
    }
    debug_assert!(dst_end <= src_begin);
    if dst_end == src_begin {
        dst_end = src_range.end;
    } else {
        while src_begin != src_range.end {
            move_node(nodes, dst_end, src_begin);
            dst_end += 1;
            src_begin += 1;
        }
    }
    Ok(dst_end)
}

fn move_node(nodes: &mut [SupportElementMerging], dst: usize, src: usize) {
    let source = &mut nodes[src];
    let moved = SupportElementMerging {
        state: source.state,
        parents: mem::take(&mut source.parents),
        areas: mem::take(&mut source.areas),
        bbox: source.bbox,
    };
    nodes[dst] = moved;
}
