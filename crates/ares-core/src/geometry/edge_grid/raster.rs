use super::super::{ClipperError, Coord, Point};

#[derive(Clone, Copy)]
pub(crate) struct RasterGrid {
    pub(crate) bounds_min: Point,
    pub(crate) resolution: Coord,
    pub(crate) rows: usize,
    pub(crate) cols: usize,
}

pub(crate) fn visit_line(
    grid: RasterGrid,
    p1: Point,
    p2: Point,
    mut visitor: impl FnMut(usize, usize) -> bool,
) -> Result<(), ClipperError> {
    let resolution = i128::from(grid.resolution);
    let x1 = i128::from(p1.x()) - i128::from(grid.bounds_min.x());
    let y1 = i128::from(p1.y()) - i128::from(grid.bounds_min.y());
    let x2 = i128::from(p2.x()) - i128::from(grid.bounds_min.x());
    let y2 = i128::from(p2.y()) - i128::from(grid.bounds_min.y());
    let mut walker = Walker {
        grid,
        col: x1 / resolution,
        row: y1 / resolution,
        end_col: x2 / resolution,
        end_row: y2 / resolution,
        visitor: &mut visitor,
    };
    if walker.visit_or_done() {
        return Ok(());
    }

    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    match (x1 < x2, y1 < y2) {
        (true, true) => walk_positive_positive(&mut walker, x1, y1, dx, dy),
        (true, false) => walk_positive_nonpositive(&mut walker, x1, y1, dx, dy),
        (false, true) => walk_nonpositive_positive(&mut walker, x1, y1, dx, dy),
        (false, false) => walk_nonpositive_nonpositive(&mut walker, x1, y1, dx, dy),
    }
}

struct Walker<'a, Visitor> {
    grid: RasterGrid,
    col: i128,
    row: i128,
    end_col: i128,
    end_row: i128,
    visitor: &'a mut Visitor,
}

impl<Visitor> Walker<'_, Visitor>
where
    Visitor: FnMut(usize, usize) -> bool,
{
    fn visit_or_done(&mut self) -> bool {
        let (Ok(row), Ok(col)) = (usize::try_from(self.row), usize::try_from(self.col)) else {
            return true;
        };
        row >= self.grid.rows
            || col >= self.grid.cols
            || !(self.visitor)(row, col)
            || (self.col == self.end_col && self.row == self.end_row)
    }
}

fn walk_positive_positive<Visitor>(
    walker: &mut Walker<'_, Visitor>,
    x1: i128,
    y1: i128,
    dx: i128,
    dy: i128,
) -> Result<(), ClipperError>
where
    Visitor: FnMut(usize, usize) -> bool,
{
    let resolution = i128::from(walker.grid.resolution);
    let mut ex = product((walker.col + 1) * resolution - x1, dy)?;
    let mut ey = product((walker.row + 1) * resolution - y1, dx)?;
    loop {
        match ex.cmp(&ey) {
            std::cmp::Ordering::Less => {
                ey -= ex;
                ex = product(dy, resolution)?;
                walker.col += 1;
            }
            std::cmp::Ordering::Equal => {
                ex = product(dy, resolution)?;
                ey = product(dx, resolution)?;
                walker.col += 1;
                walker.row += 1;
            }
            std::cmp::Ordering::Greater => {
                ex -= ey;
                ey = product(dx, resolution)?;
                walker.row += 1;
            }
        }
        if walker.visit_or_done() {
            return Ok(());
        }
    }
}

fn walk_positive_nonpositive<Visitor>(
    walker: &mut Walker<'_, Visitor>,
    x1: i128,
    y1: i128,
    dx: i128,
    dy: i128,
) -> Result<(), ClipperError>
where
    Visitor: FnMut(usize, usize) -> bool,
{
    let resolution = i128::from(walker.grid.resolution);
    let mut ex = product((walker.col + 1) * resolution - x1, dy)?;
    let mut ey = product(y1 - walker.row * resolution, dx)?;
    loop {
        if ex <= ey {
            ey -= ex;
            ex = product(dy, resolution)?;
            walker.col += 1;
        } else {
            ex -= ey;
            ey = product(dx, resolution)?;
            walker.row -= 1;
        }
        if walker.visit_or_done() {
            return Ok(());
        }
    }
}

fn walk_nonpositive_positive<Visitor>(
    walker: &mut Walker<'_, Visitor>,
    x1: i128,
    y1: i128,
    dx: i128,
    dy: i128,
) -> Result<(), ClipperError>
where
    Visitor: FnMut(usize, usize) -> bool,
{
    let resolution = i128::from(walker.grid.resolution);
    let mut ex = product(x1 - walker.col * resolution, dy)?;
    let mut ey = product((walker.row + 1) * resolution - y1, dx)?;
    loop {
        if ex < ey {
            ey -= ex;
            ex = product(dy, resolution)?;
            walker.col -= 1;
        } else {
            ex -= ey;
            ey = product(dx, resolution)?;
            walker.row += 1;
        }
        if walker.visit_or_done() {
            return Ok(());
        }
    }
}

fn walk_nonpositive_nonpositive<Visitor>(
    walker: &mut Walker<'_, Visitor>,
    x1: i128,
    y1: i128,
    dx: i128,
    dy: i128,
) -> Result<(), ClipperError>
where
    Visitor: FnMut(usize, usize) -> bool,
{
    let resolution = i128::from(walker.grid.resolution);
    let mut ex = product(x1 - walker.col * resolution, dy)?;
    let mut ey = product(y1 - walker.row * resolution, dx)?;
    loop {
        match ex.cmp(&ey) {
            std::cmp::Ordering::Less => {
                ey -= ex;
                ex = product(dy, resolution)?;
                walker.col -= 1;
            }
            std::cmp::Ordering::Equal => {
                if dx > 0 {
                    ex = product(dy, resolution)?;
                    walker.col -= 1;
                }
                if dy > 0 {
                    ey = product(dx, resolution)?;
                    walker.row -= 1;
                }
            }
            std::cmp::Ordering::Greater => {
                ex -= ey;
                ey = product(dx, resolution)?;
                walker.row -= 1;
            }
        }
        if walker.visit_or_done() {
            return Ok(());
        }
    }
}

fn product(lhs: i128, rhs: i128) -> Result<i128, ClipperError> {
    lhs.checked_mul(rhs)
        .ok_or(ClipperError::CoordinateOutOfRange)
}
