//! NLopt 2.5.0 `nldrmd_minimize_` (`src/algs/neldermead/nldrmd.c:96-290`): bound-constrained
//! Nelder-Mead as subplex's inner solver, ending when the simplex diameter shrinks by `psi`.

use super::{Outcome, Stopping};

/// The arguments `nldrmd_minimize_` and `sbplx_minimize` share: the objective, the `(lb, ub)`
/// bounds, the best point and value so far, and the stop criteria.
pub(super) struct Problem<'a> {
    pub(super) f: &'a mut dyn FnMut(&[f64]) -> f64,
    pub(super) bounds: (&'a [f64], &'a [f64]),
    pub(super) x: &'a mut [f64],
    pub(super) minf: &'a mut f64,
    pub(super) stop: &'a mut Stopping,
}

/// Strategy constants (`nldrmd.c:36`).
const ALPHA: f64 = 1.0;
const BETA: f64 = 0.5;
const GAMMA: f64 = 2.0;
const DELTA: f64 = 0.5;

/// `close`: equal to floating-point precision.
fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-13 * (a.abs() + b.abs())
}

/// `reflectpt`: `c + scale * (c - xold)` pinned to the bounds; false when it coincides with
/// `c` or `xold`.
fn reflect(
    xnew: &mut [f64],
    (c, xold): (&[f64], &[f64]),
    scale: f64,
    (lb, ub): (&[f64], &[f64]),
) -> bool {
    let (mut equal_c, mut equal_old) = (true, true);
    for i in 0..xnew.len() {
        let mut value = c[i] + scale * (c[i] - xold[i]);
        if value < lb[i] {
            value = lb[i];
        }
        if value > ub[i] {
            value = ub[i];
        }
        equal_c = equal_c && close(value, c[i]);
        equal_old = equal_old && close(value, xold[i]);
        xnew[i] = value;
    }
    !(equal_c || equal_old)
}

/// The simplex rows `[f, x...]`, ordered by f and then by row (the red-black tree's key order,
/// whose tie-breaker is the row address).
struct Simplex {
    pts: Vec<Vec<f64>>,
}

impl Simplex {
    fn order(&self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.pts.len()).collect();
        order.sort_by(|&a, &b| self.pts[a][0].total_cmp(&self.pts[b][0]).then(a.cmp(&b)));
        order
    }

    fn row(value: f64, point: &[f64]) -> Vec<f64> {
        std::iter::once(value)
            .chain(point.iter().copied())
            .collect()
    }
}

impl Problem<'_> {
    /// `CHECK_EVAL`: counts the evaluation and keeps the best point; `Some` ends the search.
    fn check(&mut self, xc: &[f64], fc: f64) -> Option<Outcome> {
        self.stop.nevals += 1;
        if fc <= *self.minf {
            *self.minf = fc;
            self.x.copy_from_slice(xc);
        }
        self.stop.evals().then_some(Outcome::MaxEval)
    }

    /// Nelder-Mead from `x` with the initial steps `xstep`, ending when the simplex diameter
    /// shrinks by `psi`. Returns the outcome and the last simplex's high minus low value.
    pub(super) fn minimize(&mut self, xstep: &[f64], psi: f64) -> (Outcome, f64) {
        let n = self.x.len();
        let (lb, ub) = self.bounds;
        let mut fdiff = f64::INFINITY;
        let mut simplex = Simplex {
            pts: vec![Simplex::row(*self.minf, self.x)],
        };
        for i in 0..n {
            let mut point = self.x.to_vec();
            point[i] = vertex(self.x[i], xstep[i], lb[i], ub[i]);
            if close(point[i], self.x[i]) {
                return (Outcome::Failure, fdiff);
            }
            let value = (self.f)(&point);
            simplex.pts.push(Simplex::row(value, &point));
            if let Some(outcome) = self.check(&point, value) {
                return (outcome, fdiff);
            }
        }
        let mut init_diam = 0.0;
        let mut c = vec![0.0; n];
        let mut xcur = vec![0.0; n];
        loop {
            let order = simplex.order();
            let (low, high) = (order[0], order[n]);
            let (fl, xl) = (simplex.pts[low][0], simplex.pts[low][1..].to_vec());
            let xh = simplex.pts[high][1..].to_vec();
            let fh = simplex.pts[high][0];
            fdiff = fh - fl;
            let diam: f64 = xl.iter().zip(&xh).map(|(l, h)| (l - h).abs()).sum();
            if init_diam == 0.0 {
                init_diam = diam;
            }
            centroid(&simplex, high, &mut c);
            if diam < psi * init_diam {
                return (Outcome::XtolReached, fdiff);
            }
            if !reflect(&mut xcur, (&c, &xh), ALPHA, self.bounds) {
                return (Outcome::XtolReached, fdiff);
            }
            let fr = (self.f)(&xcur);
            if let Some(outcome) = self.check(&xcur, fr) {
                return (outcome, fdiff);
            }
            let second = simplex.pts[order[n - 1]][0];
            let step = if fr < fl {
                // A new best point: expand.
                self.expand((&c, &xh), &xcur, fr).map(Some)
            } else if fr < second {
                Ok(Some((xcur.clone(), fr)))
            } else {
                // A new worst point: contract.
                self.contract(&mut simplex, (low, &xl), (&c, &xh, fh, fr), &mut xcur)
                    .map(|accepted| accepted.map(|fc| (xcur.clone(), fc)))
            };
            match step {
                Err(outcome) => return (outcome, fdiff),
                // A failed contraction shrank the simplex.
                Ok(None) => {}
                Ok(Some((point, value))) => simplex.pts[high] = Simplex::row(value, &point),
            }
        }
    }

    /// The expansion past a new best point `xcur` valued `fr` (`nldrmd.c:220-232`): the better
    /// of the expanded point and `xcur`, or `Err` when the search ends.
    fn expand(
        &mut self,
        (c, xh): (&[f64], &[f64]),
        xcur: &[f64],
        fr: f64,
    ) -> Result<(Vec<f64>, f64), Outcome> {
        let mut xe = xh.to_vec();
        if !reflect(&mut xe, (c, xh), GAMMA, self.bounds) {
            return Err(Outcome::XtolReached);
        }
        let fe = (self.f)(&xe);
        if let Some(outcome) = self.check(&xe, fe) {
            return Err(outcome);
        }
        Ok(if fe >= fr {
            (xcur.to_vec(), fr)
        } else {
            (xe, fe)
        })
    }

    /// The contraction towards the centroid `c` (`nldrmd.c:241-275`): `Some` value of an
    /// accepted point in `xcur`, `None` after a failed one shrank the simplex, `Err` when the
    /// search ends.
    fn contract(
        &mut self,
        simplex: &mut Simplex,
        (low, xl): (usize, &[f64]),
        (c, xh, fh, fr): (&[f64], &[f64], f64, f64),
        xcur: &mut [f64],
    ) -> Result<Option<f64>, Outcome> {
        if !reflect(
            xcur,
            (c, xh),
            if fh <= fr { -BETA } else { BETA },
            self.bounds,
        ) {
            return Err(Outcome::XtolReached);
        }
        let fc = (self.f)(xcur);
        if let Some(outcome) = self.check(xcur, fc) {
            return Err(outcome);
        }
        if fc < fr && fc < fh {
            return Ok(Some(fc));
        }
        match self.shrink(simplex, (low, xl)) {
            Some(outcome) => Err(outcome),
            None => Ok(None),
        }
    }

    /// Halves every vertex's distance to the best one; `Some` ends the search.
    fn shrink(&mut self, simplex: &mut Simplex, (low, xl): (usize, &[f64])) -> Option<Outcome> {
        for row in (0..simplex.pts.len()).filter(|row| *row != low) {
            let old = simplex.pts[row][1..].to_vec();
            let mut shrunk = old.clone();
            if !reflect(&mut shrunk, (xl, &old), -DELTA, self.bounds) {
                return Some(Outcome::XtolReached);
            }
            let value = (self.f)(&shrunk);
            simplex.pts[row] = Simplex::row(value, &shrunk);
            if let Some(outcome) = self.check(&shrunk, value) {
                return Some(outcome);
            }
        }
        None
    }
}

/// The starting simplex vertex along one axis: `x + step` pinned to the bounds, or the other
/// way when a bound is too close (`nldrmd.c:129-148`).
fn vertex(x: f64, step: f64, lb: f64, ub: f64) -> f64 {
    let mut value = x + step;
    if value > ub {
        value = if ub - x > step.abs() * 0.1 {
            ub
        } else {
            x - step.abs()
        };
    }
    if value >= lb {
        return value;
    }
    if x - lb > step.abs() * 0.1 {
        return lb;
    }
    value = x + step.abs();
    if value > ub {
        value = 0.5 * ((if ub - x > x - lb { ub } else { lb }) + x);
    }
    value
}

/// The centroid of every vertex but the highest.
fn centroid(simplex: &Simplex, high: usize, c: &mut [f64]) {
    c.fill(0.0);
    for (_, pt) in simplex
        .pts
        .iter()
        .enumerate()
        .filter(|(row, _)| *row != high)
    {
        add(c, &pt[1..]);
    }
    let ninv = 1.0 / c.len() as f64;
    c.iter_mut().for_each(|value| *value *= ninv);
}

fn add(sum: &mut [f64], values: &[f64]) {
    sum.iter_mut()
        .zip(values)
        .for_each(|(sum, value)| *sum += value);
}
