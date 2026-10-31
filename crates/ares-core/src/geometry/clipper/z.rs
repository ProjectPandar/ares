use super::{ClipOperation, backend};
use crate::geometry::{Point, clipper2 as c2};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::geometry) struct KernelPoint {
    pub(in crate::geometry) xy: Point,
    pub(in crate::geometry) z: i64,
}
pub(in crate::geometry) type ZPath = Vec<KernelPoint>;
impl KernelPoint {
    pub(in crate::geometry) const fn new(x: i64, y: i64, z: i64) -> Self {
        Self {
            xy: Point::new(x, y),
            z,
        }
    }
    pub(in crate::geometry) const fn x(self) -> i64 {
        self.xy.x()
    }
    pub(in crate::geometry) const fn y(self) -> i64 {
        self.xy.y()
    }
    pub(in crate::geometry) const fn full_eq(self, other: Self) -> bool {
        self.x() == other.x() && self.y() == other.y() && self.z == other.z
    }
    pub(in crate::geometry) fn full_cmp(self, other: Self) -> std::cmp::Ordering {
        (self.x(), self.y(), self.z).cmp(&(other.x(), other.y(), other.z))
    }
}

// ClipperZUtils.cpp:48–80. Callback suppression belongs to the wrapper; the
// engine has already inherited subject-priority Z at coincident input vertices.
pub(in crate::geometry) fn clip_zpaths(
    subject: &[ZPath],
    (open, preserve_collinear): (bool, bool),
    clip: &[ZPath],
    kind: ClipOperation,
    mut fill: impl FnMut(&c2::Point64, &c2::Point64, &c2::Point64, &c2::Point64, &mut c2::Point64)
    + 'static,
) -> Vec<ZPath> {
    let convert = |paths: &[ZPath]| {
        paths
            .iter()
            .map(|path| {
                path.iter()
                    .map(|p| c2::Point64::new_z(p.x(), p.y(), p.z))
                    .collect()
            })
            .collect()
    };
    let mut clipper = c2::Clipper64::new();
    clipper.set_preserve_collinear(preserve_collinear);
    clipper.set_z_callback(move |a, b, c, d, point| {
        if [a, b, c, d]
            .iter()
            .any(|p| p.x == point.x && p.y == point.y)
        {
            return;
        }
        let mut zpoint = *point;
        fill(a, b, c, d, &mut zpoint);
        point.z = zpoint.z;
    });
    if open {
        clipper.add_open_subject(&convert(subject));
    } else {
        clipper.add_subject(&convert(subject));
    }
    clipper.add_clip(&convert(clip));
    let mut closed = Vec::new();
    let mut opened = Vec::new();
    clipper.execute(
        backend::operation(kind),
        c2::FillRule::NonZero,
        &mut closed,
        Some(&mut opened),
    );
    closed.extend(opened);
    closed
        .into_iter()
        .map(|path| {
            path.into_iter()
                .map(|p| KernelPoint::new(p.x, p.y, p.z))
                .collect()
        })
        .collect()
}

// ClipperZUtils.hpp:ClipperZIntersectionVisitor, used by RegionExpansion.
pub(in crate::geometry) fn clip_pairs(
    subject: &[ZPath],
    clip: &[ZPath],
    kind: ClipOperation,
) -> (Vec<ZPath>, Vec<(i64, i64)>) {
    use std::{cell::RefCell, rc::Rc};
    let pairs = Rc::new(RefCell::new(Vec::new()));
    let captured = Rc::clone(&pairs);
    let paths = clip_zpaths(
        subject,
        (true, false),
        clip,
        kind,
        move |a, b, c, d, point| {
            let mut ids = [a.z, b.z, c.z, d.z];
            ids.sort_unstable();
            if ids[0] == ids[3] {
                point.z = ids[0];
            } else {
                let second = *ids.iter().find(|&&id| id != ids[0]).unwrap();
                debug_assert!(ids.iter().all(|&id| id == ids[0] || id == second));
                let mut pairs = captured.borrow_mut();
                pairs.push((ids[0], second));
                point.z = -(pairs.len() as i64);
            }
        },
    );
    let pairs = Rc::try_unwrap(pairs).unwrap().into_inner();
    (paths, pairs)
}
