//! ClipperUtils.cpp:904-928 tile_expolygons.
use crate::geometry::{BoundingBox, ExPolygon};

pub(super) struct Tile {
    pub(super) bounds: Option<BoundingBox>,
    pub(super) members: Vec<usize>,
}

pub(super) fn tiles(polygons: &[ExPolygon], per_tile: usize) -> Vec<Tile> {
    let bounds = polygons
        .iter()
        .map(|p| BoundingBox::from_expolygon(p).unwrap())
        .collect::<Vec<_>>();
    let Some(mut extent) = bounds.first().copied() else {
        return Vec::new();
    };
    for bounds in &bounds[1..] {
        extent.merge_point(bounds.min());
        extent.merge_point(bounds.max());
    }
    let count = ((polygons.len() as f64 / per_tile.max(1) as f64).sqrt() as i64).clamp(1, 32);
    let width = ((extent.max().x() - extent.min().x()) / count + 1).max(1);
    let height = ((extent.max().y() - extent.min().y()) / count + 1).max(1);
    let mut tiles = (0..count * count)
        .map(|_| Tile {
            bounds: None,
            members: Vec::new(),
        })
        .collect::<Vec<_>>();
    for (index, bounds) in bounds.into_iter().enumerate() {
        let center = bounds.center();
        let x = ((center.x() - extent.min().x()) / width).clamp(0, count - 1);
        let y = ((center.y() - extent.min().y()) / height).clamp(0, count - 1);
        let tile = &mut tiles[(y * count + x) as usize];
        tile.members.push(index);
        match &mut tile.bounds {
            Some(extent) => {
                extent.merge_point(bounds.min());
                extent.merge_point(bounds.max());
            }
            extent @ None => *extent = Some(bounds),
        }
    }
    tiles.retain(|tile| !tile.members.is_empty());
    tiles
}
