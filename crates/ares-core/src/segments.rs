#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Point2 {
    x: f64,
    y: f64,
}

impl Point2 {
    pub(crate) fn new(x: f64, y: f64) -> Self {
        Self {
            x: round_6(x),
            y: round_6(y),
        }
    }

    pub(crate) const fn x(&self) -> f64 {
        self.x
    }

    pub(crate) const fn y(&self) -> f64 {
        self.y
    }
}

fn round_6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}
