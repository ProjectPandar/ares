//! `FillLightning::DistanceField` (`src/libslic3r/Fill/Lightning/DistanceField.cpp`):
//! a grid of `UnsupportedCell`s sampling the overhang region at `radius/6`
//! spacing, each carrying its distance to the outline boundary. `update` marks
//! cells newly supported by a branch (circle at the leaf + a rotated rectangle
//! back to the parent). Used by `Layer::generateNewTrees` to seed and clear
//! unsupported locations.

use crate::fill::multiline::vline::sample_grid_pattern;
use crate::geometry::{
    BoundingBox, ClipperError, Coord, ExPolygon, FillRule, JoinType, Point, Polygon, offset2_ex,
    union_ex,
};

/// `radius_per_cell_size` — cell size = supporting radius / 6 (DistanceField.cpp:14).
const RADIUS_PER_CELL_SIZE: Coord = 6;
/// `prime_for_hash` for the stable ordering tie-break (DistanceField.cpp).
const PRIME_FOR_HASH: i64 = 191;

/// `PointHash` (`Point.hpp:505-508`) — `(89*31 + x)*31 + y`.
pub(super) fn point_hash(p: Point) -> u64 {
    ((89 * 31 + p.x()) * 31 + p.y()) as u64
}

/// `Line::distance_to_squared` — squared distance from `point` to segment `a-b`.
fn segment_distance_squared(point: Point, a: Point, b: Point) -> f64 {
    let (px, py) = (point.x() as f64, point.y() as f64);
    let (ax, ay) = (a.x() as f64, a.y() as f64);
    let (bx, by) = (b.x() as f64, b.y() as f64);
    let (dx, dy) = (bx - ax, by - ay);
    let len2 = dx * dx + dy * dy;
    if len2 == 0.0 {
        let (ex, ey) = (px - ax, py - ay);
        return ex * ex + ey * ey;
    }
    let t = (((px - ax) * dx + (py - ay) * dy) / len2).clamp(0.0, 1.0);
    let (ex, ey) = (px - (ax + t * dx), py - (ay + t * dy));
    ex * ex + ey * ey
}

/// `DistanceField::UnsupportedCell` — a sampled cell with its boundary distance.
#[derive(Clone, Copy)]
pub(super) struct UnsupportedCell {
    pub(super) loc: Point,
    pub(super) dist_to_boundary: Coord,
}

/// `DistanceField::UnsupportedPointsGrid` — flat cell→index grid for lookup.
struct UnsupportedPointsGrid {
    size: usize,
    grid_range_min: Point,
    grid_range_max: Point,
    grid_size: Point,
    data: Vec<usize>,
    data_erased: Vec<bool>,
}

impl UnsupportedPointsGrid {
    const NONE: usize = usize::MAX;

    fn initialize(cells: &[UnsupportedCell], to_grid: impl Fn(Point) -> Point) -> Self {
        if cells.is_empty() {
            return Self {
                size: 0,
                grid_range_min: Point::new(0, 0),
                grid_range_max: Point::new(0, 0),
                grid_size: Point::new(0, 0),
                data: Vec::new(),
                data_erased: Vec::new(),
            };
        }
        let mut bbox = BoundingBox::new(to_grid(cells[0].loc), to_grid(cells[0].loc));
        for cell in cells {
            bbox.merge_point(to_grid(cell.loc));
        }
        let grid_size = Point::new(
            bbox.max().x() - bbox.min().x() + 1,
            bbox.max().y() - bbox.min().y() + 1,
        );
        let area = (grid_size.x() * grid_size.y()).max(0) as usize;
        let mut grid = Self {
            size: cells.len(),
            grid_range_min: bbox.min(),
            grid_range_max: bbox.max(),
            grid_size,
            data: vec![Self::NONE; area],
            data_erased: vec![true; area],
        };
        for (idx, cell) in cells.iter().enumerate() {
            let flat = grid.flat_idx(to_grid(cell.loc));
            grid.data[flat] = idx;
            grid.data_erased[flat] = false;
        }
        grid
    }

    fn flat_idx(&self, grid_addr: Point) -> usize {
        let (ox, oy) = (
            grid_addr.x() - self.grid_range_min.x(),
            grid_addr.y() - self.grid_range_min.y(),
        );
        (self.grid_size.x() * oy + ox) as usize
    }

    fn contains(&self, grid_addr: Point) -> bool {
        grid_addr.x() >= self.grid_range_min.x()
            && grid_addr.x() <= self.grid_range_max.x()
            && grid_addr.y() >= self.grid_range_min.y()
            && grid_addr.y() <= self.grid_range_max.y()
    }

    fn find_cell_idx(&self, grid_addr: Point) -> usize {
        if !self.contains(grid_addr) {
            return Self::NONE;
        }
        let flat = self.flat_idx(grid_addr);
        if !self.data_erased[flat] {
            self.data[flat]
        } else {
            Self::NONE
        }
    }

    fn mark_erased(&mut self, grid_addr: Point) {
        if !self.contains(grid_addr) {
            return;
        }
        let flat = self.flat_idx(grid_addr);
        if !self.data_erased[flat] {
            self.data_erased[flat] = true;
            self.size = self.size.saturating_sub(1);
        }
    }
}

/// `FillLightning::DistanceField`.
pub(super) struct DistanceField {
    cell_size: Coord,
    supporting_radius: Coord,
    supporting_radius2: i64,
    unsupported_points: Vec<UnsupportedCell>,
    unsupported_points_erased: Vec<bool>,
    unsupported_points_bbox: BoundingBox,
    unsupported_points_grid: UnsupportedPointsGrid,
}

impl DistanceField {
    /// `DistanceField::DistanceField` — sample the overhang with a regular grid
    /// (`offset2_ex(union_ex(overhang), -cs/2, cs/2)` then `sample_grid_pattern`),
    /// set each cell's `dist_to_boundary`, sort by the prime-hash, build the grid.
    pub(super) fn new(
        radius: Coord,
        current_outlines_bbox: BoundingBox,
        current_overhang: &[Polygon],
    ) -> Result<Self, ClipperError> {
        let cell_size = radius / RADIUS_PER_CELL_SIZE;
        let supporting_radius2 = radius * radius;
        let overhang_bbox =
            BoundingBox::from_polygons(current_overhang).unwrap_or(current_outlines_bbox);

        // `union_ex` default `NonZero`; `offset2_ex` default `jtMiter` +
        // miterLimit 3.0 (ClipperUtils.hpp).
        let unioned = union_ex(current_overhang, FillRule::NonZero)?;
        let expanded = offset2_ex(
            &unioned,
            -(cell_size / 2) as f32,
            (cell_size / 2) as f32,
            JoinType::Miter,
            3.0,
        )?;

        let mut unsupported_points: Vec<UnsupportedCell> = Vec::new();
        for expoly in &expanded {
            // The full polygon list (contour + holes) for the rasterizer.
            let mut all: Vec<Polygon> = vec![expoly.contour().clone()];
            all.extend(expoly.holes().iter().cloned());
            let sampled =
                sample_grid_pattern(&all, cell_size, (overhang_bbox.min(), overhang_bbox.max()));
            for sp in sampled {
                // Min squared distance to this expolygon's boundary.
                let d2 = boundary_distance_squared(expoly, sp);
                unsupported_points.push(UnsupportedCell {
                    loc: sp,
                    dist_to_boundary: d2.sqrt() as Coord,
                });
            }
        }

        // `std::stable_sort` with the single upstream comparator
        // (DistanceField.cpp:72-77): if `abs(b.dist - a.dist) > radius`, order
        // by dist_to_boundary ascending; else by the prime-hash of the loc.
        // The comparator is non-transitive: both stability and libstdc++'s
        // seven-cell insertion runs / half-buffer merge sequence matter.
        crate::geometry::stable_sort_by(&mut unsupported_points, |a, b| {
            if (b.dist_to_boundary - a.dist_to_boundary).unsigned_abs() > radius as u64 {
                a.dist_to_boundary < b.dist_to_boundary
            } else {
                (point_hash(a.loc) % PRIME_FOR_HASH as u64)
                    < (point_hash(b.loc) % PRIME_FOR_HASH as u64)
            }
        });

        let unsupported_points_erased = vec![false; unsupported_points.len()];
        let unsupported_points_grid = {
            let cs = cell_size;
            let bbox_min = current_outlines_bbox.min();
            UnsupportedPointsGrid::initialize(&unsupported_points, move |p| {
                Point::new((p.x() - bbox_min.x()) / cs, (p.y() - bbox_min.y()) / cs)
            })
        };

        Ok(Self {
            cell_size,
            supporting_radius: radius,
            supporting_radius2,
            unsupported_points,
            unsupported_points_erased,
            unsupported_points_bbox: current_outlines_bbox,
            unsupported_points_grid,
        })
    }

    /// `tryGetNextPoint` — the next non-erased unsupported point's index+loc.
    pub(super) fn try_get_next_point(&self, start_idx: usize) -> Option<(usize, Point)> {
        (start_idx..self.unsupported_points.len())
            .find(|&i| !self.unsupported_points_erased[i])
            .map(|i| (i, self.unsupported_points[i].loc))
    }

    fn to_grid_point(&self, p: Point) -> Point {
        Point::new(
            (p.x() - self.unsupported_points_bbox.min().x()) / self.cell_size,
            (p.y() - self.unsupported_points_bbox.min().y()) / self.cell_size,
        )
    }

    #[expect(
        clippy::wrong_self_convention,
        reason = "mirrors upstream `DistanceField::from_grid_point`"
    )]
    fn from_grid_point(&self, p: Point) -> Point {
        Point::new(
            p.x() * self.cell_size + self.unsupported_points_bbox.min().x(),
            p.y() * self.cell_size + self.unsupported_points_bbox.min().y(),
        )
    }

    /// `DistanceField::update` — mark cells supported by the new branch
    /// (circle at `added_leaf` ∪ rotated rectangle `to_node`→`added_leaf`).
    /// Upstream `d < -1 || d > 1` is false for a NaN extent; `contains` is not.
    pub(super) fn update(&mut self, to_node: Point, added_leaf: Point) {
        let (vx, vy) = (
            (added_leaf.x() - to_node.x()) as f64,
            (added_leaf.y() - to_node.y()) as f64,
        );
        // No zero-length guard upstream: `extent` goes NaN, the saturating
        // cast keeps the grid at the leaf circle, and the NaN extent test
        // below never rejects — only the radius-gated erase applies.
        let l2 = vx * vx + vy * vy;
        let ex = -vy * self.supporting_radius as f64 / l2.sqrt();
        let ey = vx * self.supporting_radius as f64 / l2.sqrt();

        let diag = self.supporting_radius;
        let mut grid_min = Point::new(added_leaf.x() - diag, added_leaf.y() - diag);
        let mut grid_max = Point::new(added_leaf.x() + diag, added_leaf.y() + diag);
        let iext = Point::new(ex as Coord, ey as Coord);
        let sub = |a: Point, b: Point| Point::new(a.x() - b.x(), a.y() - b.y());
        let add = |a: Point, b: Point| Point::new(a.x() + b.x(), a.y() + b.y());
        for p in [
            sub(to_node, iext),
            add(to_node, iext),
            sub(added_leaf, iext),
            add(added_leaf, iext),
        ] {
            grid_min = Point::new(grid_min.x().min(p.x()), grid_min.y().min(p.y()));
            grid_max = Point::new(grid_max.x().max(p.x()), grid_max.y().max(p.y()));
        }
        // Clip to the unsupported-points bbox.
        let bb = self.unsupported_points_bbox;
        grid_min = Point::new(
            grid_min.x().max(bb.min().x()),
            grid_min.y().max(bb.min().y()),
        );
        grid_max = Point::new(
            grid_max.x().min(bb.max().x()),
            grid_max.y().min(bb.max().y()),
        );
        let gmin = self.to_grid_point(grid_min);
        let gmax = self.to_grid_point(grid_max);

        for (gy, gx) in
            (gmin.y()..=gmax.y()).flat_map(|gy| (gmin.x()..=gmax.x()).map(move |gx| (gy, gx)))
        {
            let grid_addr = Point::new(gx, gy);
            let grid_loc = self.from_grid_point(grid_addr);
            let inside_circle = {
                let dx = grid_loc.x() - added_leaf.x();
                let dy = grid_loc.y() - added_leaf.y();
                dx * dx + dy * dy <= self.supporting_radius2
            };
            let reject_grid_corner = if !inside_circle {
                // Rotated-rectangle test.
                let vxx = (grid_loc.x() - to_node.x()) as f64;
                let vyy = (grid_loc.y() - to_node.y()) as f64;
                let d = vx * vxx + vy * vyy;
                let de = ex * vxx + ey * vyy;
                // Outside `[0, l2]` falls through to the erase check.
                (0.0..=l2).contains(&d) && (de < -1.0 || de > 1.0)
            } else {
                false
            };
            if reject_grid_corner {
                continue;
            }
            let cell_idx = self.unsupported_points_grid.find_cell_idx(grid_addr);
            if cell_idx == UnsupportedPointsGrid::NONE {
                continue;
            }
            let cell = self.unsupported_points[cell_idx];
            let dx = cell.loc.x() - added_leaf.x();
            let dy = cell.loc.y() - added_leaf.y();
            if dx * dx + dy * dy <= self.supporting_radius2 {
                self.unsupported_points_erased[cell_idx] = true;
                self.unsupported_points_grid.mark_erased(grid_addr);
            }
        }
    }
}

fn boundary_distance_squared(expoly: &ExPolygon, sp: Point) -> f64 {
    let mut d2 = f64::MAX;
    let mut polys: Vec<&Polygon> = vec![expoly.contour()];
    polys.extend(expoly.holes().iter());
    for contour in polys {
        let pts = contour.points();
        if pts.len() > 2 {
            let mut prev = *pts.last().unwrap();
            for &p2 in pts {
                d2 = d2.min(segment_distance_squared(sp, prev, p2));
                prev = p2;
            }
        }
    }
    d2
}
