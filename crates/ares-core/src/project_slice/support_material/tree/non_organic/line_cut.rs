//! Layer-local contour intersection cache from `TreeSupport.cpp:2740-2757`.

use std::collections::HashMap;

use crate::geometry::{ClipperError, Point, Polygon, Polyline, intersection_open_polylines};

pub(super) struct LineCutCache {
    contours: Vec<Polygon>,
    cuts: HashMap<[i64; 4], bool>,
}

impl LineCutCache {
    pub(super) fn new(contours: Vec<Polygon>) -> Self {
        Self {
            contours,
            cuts: HashMap::new(),
        }
    }

    pub(super) fn is_cut(&mut self, first: Point, second: Point) -> Result<bool, ClipperError> {
        let key = [first.x(), first.y(), second.x(), second.y()];
        if let Some(&cut) = self.cuts.get(&key) {
            return Ok(cut);
        }
        let cut = self.intersects(second, first)?;
        self.cuts.insert(key, cut);
        self.cuts
            .insert([second.x(), second.y(), first.x(), first.y()], cut);
        Ok(cut)
    }

    pub(super) fn intersects(&self, first: Point, second: Point) -> Result<bool, ClipperError> {
        Ok(
            intersection_open_polylines(&[Polyline::new(vec![first, second])], &self.contours)?
                .iter()
                .any(|polyline| polyline.points().len() >= 2),
        )
    }
}
