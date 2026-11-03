//! `FillCornerSmoothing.cpp:71–191` bounded per-thread corner shapes.

use super::EPSILON;
use super::{
    curve::{self, UnitCorner},
    vector::V,
};
use std::{cell::RefCell, collections::BTreeMap};

struct Shapes {
    corners: BTreeMap<(u64, usize), UnitCorner>,
    peaks: BTreeMap<u64, f64>,
}

thread_local! {
    static SHAPES: RefCell<Shapes> = const { RefCell::new(Shapes {
        corners: BTreeMap::new(), peaks: BTreeMap::new(),
    }) };
}

fn key(cosine: f64) -> u64 {
    if cosine == 0.0 { 0 } else { cosine.to_bits() }
}

impl Shapes {
    fn peak(&mut self, cosine: f64) -> f64 {
        let key = key(cosine);
        if let Some(&peak) = self.peaks.get(&key) {
            return peak;
        }
        let peak = curve::peak_curvature(cosine);
        if self.peaks.len() >= 4096 {
            self.peaks.clear();
        }
        self.peaks.insert(key, peak);
        peak
    }

    fn corner(&mut self, cosine: f64, count: usize) -> &UnitCorner {
        let key = (key(cosine), count);
        if !self.corners.contains_key(&key) {
            if self.corners.len() >= 4096 {
                self.corners.clear();
            }
            self.corners
                .insert(key, curve::tangent_polygon(cosine, count));
        }
        &self.corners[&key]
    }
}

pub(super) fn vertices(reach: f64, cosine: f64, tolerance: f64, mut emit: impl FnMut(V)) -> bool {
    let tolerance = if tolerance > 0.0 && tolerance.is_finite() {
        tolerance
    } else {
        EPSILON
    };
    SHAPES.with(|storage| {
        let mut shapes = storage.borrow_mut();
        let turn = cosine.clamp(-1.0, 1.0).acos();
        let curvature = shapes.peak(cosine) / reach;
        let most = 256.0_f64.min(turn / (2.0 * curvature * tolerance).sqrt()) as usize;
        let mut count =
            most.min((turn / (8.0 * curvature * tolerance).sqrt() - EPSILON).ceil() as usize);
        if count < 2 {
            return false;
        }
        loop {
            let deviation = reach * shapes.corner(cosine, count).deviation;
            if deviation <= tolerance || count >= most {
                break;
            }
            count = most.min(
                (count + 1).max((count as f64 * (deviation / tolerance).sqrt()).ceil() as usize),
            );
        }
        for &vertex in &shapes.corner(cosine, count).vertices {
            emit(vertex);
        }
        true
    })
}
