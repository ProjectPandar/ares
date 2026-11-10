//! MultiMaterialSegmentation.cpp:1999-2102 merge_segmented_layers.
mod locator;

use super::{math::paths, preprocess::Layer, tiles::tiles};
use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, JoinType, Polygon,
    clip_clipper_polygons_with_subject_bbox, difference_ex_polygons, offset2_ex, union_expolygons,
};

fn root(parent: &mut [usize], mut index: usize) -> usize {
    while parent[index] != index {
        parent[index] = parent[parent[index]];
        index = parent[index];
    }
    index
}

fn difference_by_piece(
    subject: &[ExPolygon],
    clip: &[Polygon],
    epsilon: i64,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let tiles = tiles(subject, 32);
    if tiles.len() <= 1 {
        return difference_ex_polygons(subject, clip);
    }
    let mut output = Vec::new();
    for tile in tiles {
        let subject = tile
            .members
            .into_iter()
            .map(|index| subject[index].clone())
            .collect::<Vec<_>>();
        let clip =
            clip_clipper_polygons_with_subject_bbox(clip, tile.bounds.unwrap().inflated(epsilon));
        output.extend(difference_ex_polygons(&subject, &clip)?);
    }
    Ok(output)
}

fn merge_layer(
    side: &[Vec<ExPolygon>],
    caps: &[Vec<Vec<ExPolygon>>],
    layer: usize,
    islands: &[ExPolygon],
    epsilon: f32,
) -> Result<Vec<Vec<ExPolygon>>, ClipperError> {
    let states = side.len();
    let mut output = vec![Vec::new(); states - 1];
    let locator = locator::IslandLocator::new(islands);
    let mut parent = (0..=islands.len()).collect::<Vec<_>>();
    // Preserve upstream piece order: nonzero sides, then every top/bottom color.
    let mut pieces = Vec::new();
    for (color, polygons) in side.iter().enumerate().skip(1) {
        pieces.extend(polygons.iter().map(|polygon| (false, color, polygon)));
    }
    for (color, layers) in caps.iter().enumerate() {
        pieces.extend(layers[layer].iter().map(|polygon| (true, color, polygon)));
    }
    let mut piece_island = Vec::with_capacity(pieces.len());
    for &(_, _, piece) in &pieces {
        let overlapped = locator.find(piece, epsilon as i64)?;
        let first = overlapped.first().copied().unwrap_or(islands.len());
        piece_island.push(first);
        for island in overlapped {
            let from = root(&mut parent, island);
            parent[from] = root(&mut parent, first);
        }
    }
    let mut bucket_of = vec![usize::MAX; parent.len()];
    let mut buckets = 0;
    for index in 0..parent.len() {
        let owner = root(&mut parent, index);
        if bucket_of[owner] == usize::MAX {
            bucket_of[owner] = buckets;
            buckets += 1;
        }
    }
    let mut sides = vec![vec![Vec::new(); states]; buckets];
    let mut tops = vec![vec![Vec::new(); states]; buckets];
    for ((is_top, color, piece), island) in pieces.into_iter().zip(piece_island) {
        let bucket = bucket_of[root(&mut parent, island)];
        if is_top {
            &mut tops[bucket][color]
        } else {
            &mut sides[bucket][color]
        }
        .push(piece.clone());
    }
    let mut merged = vec![vec![Vec::new(); states]; buckets];
    for bucket in 0..buckets {
        let tops_all = paths(tops[bucket].iter().flatten().cloned());
        for color in 1..states {
            merged[bucket][color] = if tops_all.is_empty() {
                std::mem::take(&mut sides[bucket][color])
            } else {
                difference_by_piece(&sides[bucket][color], &tops_all, epsilon as i64)?
            };
        }
    }
    for color in 1..states {
        let has_tops = !caps.is_empty() && !caps[color][layer].is_empty();
        let had_sides = merged.iter().any(|bucket| !bucket[color].is_empty());
        for bucket in 0..buckets {
            let region = &mut merged[bucket][color];
            if !has_tops {
                output[color - 1].append(region);
                continue;
            }
            region.append(&mut tops[bucket][color]);
            if had_sides && !region.is_empty() {
                *region = offset2_ex(
                    &union_expolygons(region, &[])?,
                    epsilon,
                    -epsilon,
                    JoinType::Miter,
                    3.0,
                )?;
            }
            output[color - 1].append(region);
        }
    }
    Ok(output)
}

pub(super) fn merge(
    sides: Vec<Vec<Vec<ExPolygon>>>,
    caps: Vec<Vec<Vec<ExPolygon>>>,
    input: &[Layer],
    scale: CoordinateScale,
) -> Result<Vec<Vec<Vec<ExPolygon>>>, ClipperError> {
    let epsilon = scale.scaled_delta(1e-4) as f32;
    sides
        .iter()
        .enumerate()
        .map(|(layer, side)| merge_layer(side, &caps, layer, &input[layer].input, epsilon))
        .collect()
}
