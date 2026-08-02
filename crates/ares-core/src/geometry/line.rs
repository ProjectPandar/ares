use super::Point;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ThickLine {
    pub(crate) a: Point,
    pub(crate) b: Point,
    pub(crate) a_width: f64,
    pub(crate) b_width: f64,
}

impl ThickLine {
    pub(crate) const fn new(a: Point, b: Point) -> Self {
        Self {
            a,
            b,
            a_width: 0.0,
            b_width: 0.0,
        }
    }

    pub(crate) const fn with_widths(a: Point, b: Point, a_width: f64, b_width: f64) -> Self {
        Self {
            a,
            b,
            a_width,
            b_width,
        }
    }
}
