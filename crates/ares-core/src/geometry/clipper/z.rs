use super::{Clipper, ClipperError, PathRole};
use crate::geometry::Point;

#[derive(Clone, Copy, Debug)]
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
    pub(in crate::geometry) const fn with_x(self, x: i64) -> Self {
        Self::new(x, self.y(), self.z)
    }
    pub(in crate::geometry) const fn full_eq(self, other: Self) -> bool {
        self.x() == other.x() && self.y() == other.y() && self.z == other.z
    }
    pub(in crate::geometry) fn full_cmp(self, other: Self) -> std::cmp::Ordering {
        match self.x().cmp(&other.x()) {
            std::cmp::Ordering::Equal => match self.y().cmp(&other.y()) {
                std::cmp::Ordering::Equal => self.z.cmp(&other.z),
                order => order,
            },
            order => order,
        }
    }
}

impl From<Point> for KernelPoint {
    fn from(xy: Point) -> Self {
        Self { xy, z: 0 }
    }
}

impl PartialEq for KernelPoint {
    fn eq(&self, other: &Self) -> bool {
        self.xy == other.xy
    }
}
impl Eq for KernelPoint {}

impl Clipper {
    pub(in crate::geometry) fn add_z_closed_path(
        &mut self,
        path: &[KernelPoint],
        role: PathRole,
    ) -> Result<bool, ClipperError> {
        self.add_path(path, role, true)
    }

    pub(in crate::geometry) fn add_z_open_path(
        &mut self,
        path: &[KernelPoint],
        role: PathRole,
    ) -> Result<bool, ClipperError> {
        if role == PathRole::Clip {
            return Err(ClipperError::OpenPathMustBeSubject);
        }
        self.add_path(path, role, false)
    }

    pub(super) fn set_z(
        &mut self,
        point: &mut KernelPoint,
        first: super::types::Edge,
        second: super::types::Edge,
    ) {
        if point.z != 0 {
            return;
        }
        for endpoint in [first.bottom, first.top, second.bottom, second.top] {
            if point.xy == endpoint.xy {
                point.z = endpoint.z;
                return;
            }
        }
        if let Some(mark) = self.z_fill_mark {
            point.z = mark;
            return;
        }
        if self.z_fill_source_index {
            // `Algorithm/LineSplit.cpp:19-28::cb_split_line`.
            let minimum = [first.bottom.z, first.top.z, second.bottom.z, second.top.z]
                .into_iter()
                .min()
                .expect("four edge endpoints");
            point.z = -(minimum + 1);
            return;
        }
        if self.z_fill_width {
            // PerimeterGenerator.cpp:299-301: the interpolation parameter
            // comes from Eigen 3D squaredNorm over (x, y, z) — the width
            // delta enters the edge length, and the not-yet-set intersection
            // z (= 0) enters the distance, so t is not the geometric
            // fraction and may exceed 1.
            let (start, end) = if first.bottom.z <= 0 && first.top.z <= 0 {
                (second.bottom, second.top)
            } else {
                (first.bottom, first.top)
            };
            let length_sqr = {
                let dx = (end.x() - start.x()) as f64;
                let dy = (end.y() - start.y()) as f64;
                let dz = (end.z - start.z) as f64;
                dx * dx + dy * dy + dz * dz
            };
            let dist_sqr = {
                let px = (point.x() - start.x()) as f64;
                let py = (point.y() - start.y()) as f64;
                let pz = (0 - start.z) as f64;
                px * px + py * py + pz * pz
            };
            let t = (dist_sqr / length_sqr).sqrt();
            point.z = start.z + ((end.z - start.z) as f64 * t) as i64;
            return;
        }
        let Some(table) = self.z_intersections.as_mut() else {
            return;
        };
        fill_z(
            point,
            table,
            [first.bottom.z, first.top.z, second.bottom.z, second.top.z],
        );
    }
}

fn fill_z(point: &mut KernelPoint, table: &mut Vec<(i64, i64)>, mut values: [i64; 4]) {
    for index in 1..values.len() {
        let mut current = index;
        while current > 0 && values[current] < values[current - 1] {
            values.swap(current, current - 1);
            current -= 1;
        }
    }
    let mut unique = [0_i64; 4];
    let mut count = 0;
    for value in values {
        if count == 0 || unique[count - 1] != value {
            unique[count] = value;
            count += 1;
        }
    }
    if count == 1 {
        point.z = unique[0];
    } else {
        debug_assert_eq!(count, 2);
        table.push((unique[0], unique[1]));
        point.z = -(table.len() as i64);
    }
}

#[cfg(test)]
pub(in crate::geometry) fn z_fill_for_test(values: [i64; 4]) -> (i64, Vec<(i64, i64)>) {
    let mut point = KernelPoint::new(0, 0, 0);
    let mut table = Vec::new();
    fill_z(&mut point, &mut table, values);
    (point.z, table)
}

#[cfg(test)]
pub(in crate::geometry) fn set_z_for_test(
    candidate: KernelPoint,
    endpoints: [KernelPoint; 4],
) -> (KernelPoint, Vec<(i64, i64)>) {
    use super::types::{Edge, EdgeId};

    let mut first = Edge::new(endpoints[0], PathRole::Subject, EdgeId(0), EdgeId(0));
    first.bottom = endpoints[0];
    first.top = endpoints[1];
    let mut second = Edge::new(endpoints[2], PathRole::Clip, EdgeId(0), EdgeId(0));
    second.bottom = endpoints[2];
    second.top = endpoints[3];
    let mut clipper = Clipper::new(super::ClipperOptions::default());
    clipper.z_intersections = Some(Vec::new());
    let mut candidate = candidate;
    clipper.set_z(&mut candidate, first, second);
    (candidate, clipper.z_intersections.take().unwrap())
}
