// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

pub(crate) const FLOATING_POINT_TOLERANCE: f64 = 1e-12;

/// Default arc approximation constant (1/500).
/// When arc_tolerance is 0, the calculated default arc tolerance
/// (offset_radius * arc_const) generally produces good (smooth) arc
/// approximations without producing excessively small segment lengths.
pub(crate) const ARC_CONST: f64 = 0.002;

/// Join type for path offsetting.
/// Direct port from clipper.offset.h line 19.
///
/// - `Square`: Joins are 'squared' at exactly the offset distance (more complex code)
/// - `Bevel`: Similar to Square, but offset distance varies with angle (simple & faster)
/// - `Round`: Joins are rounded (arc approximation)
/// - `Miter`: Joins extend to a point, limited by miter_limit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JoinType {
    Square,
    Bevel,
    Round,
    Miter,
}

/// End type for open path offsetting.
/// Direct port from clipper.offset.h line 23.
///
/// - `Polygon`: Offsets only one side of a closed path
/// - `Joined`: Offsets both sides of a path, with joined ends
/// - `Butt`: Offsets both sides of a path, with square blunt ends
/// - `Square`: Offsets both sides of a path, with square extended ends
/// - `Round`: Offsets both sides of a path, with round extended ends
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndType {
    Polygon,
    Joined,
    Butt,
    Square,
    Round,
}

/// Delta callback type for variable offset.
/// Called for each vertex to determine the offset delta at that point.
/// Parameters: (path, path_normals, curr_idx, prev_idx) -> delta
pub type DeltaCallback64 = Box<dyn Fn(&Path64, &PathD, usize, usize) -> f64>;

/// Find the lowest (highest y) closed path and determine its orientation.
/// Direct port from clipper.offset.cpp GetLowestClosedPathInfo.
pub(crate) fn get_lowest_closed_path_info(paths: &Paths64) -> (Option<usize>, bool) {
    let mut idx: Option<usize> = None;
    let mut bot_pt = Point64::new(i64::MAX, i64::MIN);
    let mut is_neg_area = false;

    for (i, path_i) in paths.iter().enumerate() {
        let mut a: f64 = f64::MAX;
        for pt in path_i {
            if (pt.y < bot_pt.y) || (pt.y == bot_pt.y && pt.x >= bot_pt.x) {
                continue;
            }
            if a == f64::MAX && !update_lowest_path_area(path_i, &mut a, &mut is_neg_area) {
                break;
            }
            idx = Some(i);
            bot_pt.x = pt.x;
            bot_pt.y = pt.y;
        }
    }
    (idx, is_neg_area)
}

// clipper.offset.cpp:40–71: area is evaluated only for a new lowest candidate.
fn update_lowest_path_area(path: &Path64, value: &mut f64, negative: &mut bool) -> bool {
    *value = area(path);
    if *value == 0.0 {
        return false;
    }
    *negative = *value < 0.0;
    true
}

/// Hypotenuse calculation.
/// Direct port from clipper.offset.cpp Hypot.
#[inline]
pub(crate) fn hypot_xy(x: f64, y: f64) -> f64 {
    (x * x + y * y).sqrt()
}

/// Get unit normal vector between two points.
/// Direct port from clipper.offset.cpp GetUnitNormal.
#[inline]
pub(crate) fn get_unit_normal(pt1: &Point64, pt2: &Point64) -> PointD {
    if pt1 == pt2 {
        return PointD::new(0.0, 0.0);
    }
    let dx = (pt2.x - pt1.x) as f64;
    let dy = (pt2.y - pt1.y) as f64;
    let inverse_hypot = 1.0 / hypot_xy(dx, dy);
    PointD::new(dy * inverse_hypot, -dx * inverse_hypot)
}

/// Check if a floating-point value is approximately zero.
/// Direct port from clipper.offset.cpp AlmostZero.
#[inline]
pub(crate) fn almost_zero(value: f64, epsilon: f64) -> bool {
    value.abs() < epsilon
}

/// Normalize a 2D vector to unit length.
/// Direct port from clipper.offset.cpp NormalizeVector.
#[inline]
pub(crate) fn normalize_vector(vec: &PointD) -> PointD {
    let h = hypot_xy(vec.x, vec.y);
    if almost_zero(h, 0.001) {
        return PointD::new(0.0, 0.0);
    }
    let inverse_hypot = 1.0 / h;
    PointD::new(vec.x * inverse_hypot, vec.y * inverse_hypot)
}

/// Get the average of two unit vectors, normalized.
/// Direct port from clipper.offset.cpp GetAvgUnitVector.
#[inline]
pub(crate) fn get_avg_unit_vector(vec1: &PointD, vec2: &PointD) -> PointD {
    normalize_vector(&PointD::new(vec1.x + vec2.x, vec1.y + vec2.y))
}

/// Get perpendicular offset point (returns Point64, rounds coordinates).
/// Direct port from clipper.offset.cpp GetPerpendic.
#[inline]
pub(crate) fn get_perpendic(pt: &Point64, norm: &PointD, delta: f64) -> Point64 {
    Point64 {
        x: (pt.x as f64 + norm.x * delta).round() as i64,
        y: (pt.y as f64 + norm.y * delta).round() as i64,

        z: pt.z,
    }
}

/// Get perpendicular offset point (returns PointD, no rounding).
/// Direct port from clipper.offset.cpp GetPerpendicD.
#[inline]
pub(crate) fn get_perpendic_d(pt: &Point64, norm: &PointD, delta: f64) -> PointD {
    PointD {
        x: pt.x as f64 + norm.x * delta,
        y: pt.y as f64 + norm.y * delta,

        z: pt.z,
    }
}

/// Negate all coordinates in a PathD.
/// Direct port from clipper.offset.cpp NegatePath.
#[inline]
pub(crate) fn negate_path(path: &mut PathD) {
    for pt in path.iter_mut() {
        pt.x = -pt.x;
        pt.y = -pt.y;
    }
}

/// Internal grouping of paths with shared join/end types.
/// Direct port from ClipperOffset::Group (clipper.offset.h line 35-43).
pub(crate) struct Group {
    pub(crate) paths_in: Paths64,
    pub(crate) lowest_path_idx: Option<usize>,
    pub(crate) is_reversed: bool,
    pub(crate) join_type: JoinType,
    pub(crate) end_type: EndType,
}

impl Group {
    /// Construct a new Group, stripping duplicates and determining orientation.
    /// Direct port from ClipperOffset::Group constructor (clipper.offset.cpp line 139-162).
    pub(crate) fn new(paths: &Paths64, join_type: JoinType, end_type: EndType) -> Self {
        let mut paths_in = paths.clone();
        let is_joined = end_type == EndType::Polygon || end_type == EndType::Joined;

        for p in paths_in.iter_mut() {
            strip_duplicates_path(p, is_joined);
        }

        let (lowest_path_idx, is_reversed) = if end_type == EndType::Polygon {
            let (idx, is_neg_area) = get_lowest_closed_path_info(&paths_in);
            // The lowermost path must be an outer path, so if its orientation is
            // negative, then flag the whole group as 'reversed' (will negate delta etc.)
            // as this is much more efficient than reversing every path.
            let is_reversed = idx.is_some() && is_neg_area;
            (idx, is_reversed)
        } else {
            (None, false)
        };

        Group {
            paths_in,
            lowest_path_idx,
            is_reversed,
            join_type,
            end_type,
        }
    }
}

/// Path offset (inflate/shrink) engine.
/// Direct port from ClipperOffset class (clipper.offset.h / clipper.offset.cpp).
///
/// Inflates (or shrinks) both open and closed paths using the specified join type
/// and end type. After building the raw offset, a Clipper64 union is used to
/// clean up self-intersections.
pub struct ClipperOffset {
    // Internal state
    pub(crate) delta: f64,
    pub(crate) group_delta: f64,
    pub(crate) temp_lim: f64,
    pub(crate) steps_per_rad: f64,
    pub(crate) step_sin: f64,
    pub(crate) step_cos: f64,
    pub(crate) norms: PathD,
    pub(crate) path_out: Path64,
    pub(crate) solution: Paths64,
    pub(crate) groups: Vec<Group>,
    pub(crate) join_type: JoinType,
    pub(crate) end_type: EndType,

    // User-configurable parameters
    pub(crate) miter_limit: f64,
    pub(crate) arc_tolerance: f64,
    pub(crate) preserve_collinear: bool,
    pub(crate) reverse_solution: bool,

    // Callbacks
    pub(crate) delta_callback: Option<DeltaCallback64>,

    // Rc<RefCell<..>> replaces C++'s std::bind(this): the ZCB proxy closure
    // installed on the finishing union must be 'static, so it can't borrow
    // self and instead shares ownership of the user callback.
    pub(crate) z_callback: Option<Rc<RefCell<Box<dyn ZCallback64 + 'static>>>>,
}

impl ClipperOffset {
    /// Create a new ClipperOffset with the given parameters.
    /// Direct port from ClipperOffset constructor (clipper.offset.h line 85-91).
    pub fn new(
        miter_limit: f64,
        arc_tolerance: f64,
        preserve_collinear: bool,
        reverse_solution: bool,
    ) -> Self {
        ClipperOffset {
            delta: 0.0,
            group_delta: 0.0,
            temp_lim: 0.0,
            steps_per_rad: 0.0,
            step_sin: 0.0,
            step_cos: 0.0,
            norms: PathD::new(),
            path_out: Path64::new(),
            solution: Paths64::new(),
            groups: Vec::new(),
            join_type: JoinType::Bevel,
            end_type: EndType::Polygon,
            miter_limit,
            arc_tolerance,
            preserve_collinear,
            reverse_solution,
            delta_callback: None,

            z_callback: None,
        }
    }
}

impl ClipperOffset {
    /// Create a new ClipperOffset with default parameters.
    /// miter_limit = 2.0, arc_tolerance = 0.0,
    /// preserve_collinear = false, reverse_solution = false.
    pub fn new_default() -> Self {
        Self::new(2.0, 0.0, false, false)
    }
}

impl ClipperOffset {
    /// Set the miter limit.
    pub fn set_miter_limit(&mut self, miter_limit: f64) {
        self.miter_limit = miter_limit;
    }
}
