//! The NLopt 2.5.0 subplex minimizer libnest2d's `NloptOptimizer` runs for `LN_SBPLX`
//! (`src/algs/neldermead/sbplx.c`, `nldrmd.c`, `src/util/stop.c`, the initial step of
//! `nlopt_set_default_initial_step` in `src/api/options.c:721-766`). Only the stopping
//! criteria libnest2d sets are kept: a relative score difference and an evaluation budget.

/// `nlopt_stopping` with `minf_max` `-HUGE_VAL`, zero absolute and x tolerances and no time
/// limit.
pub(super) struct Stopping {
    pub(super) ftol_rel: f64,
    pub(super) maxeval: usize,
    pub(super) nevals: usize,
}

/// `relstop` (`stop.c:33-38`) with zero absolute tolerance.
fn relstop(old: f64, new: f64, rel: f64) -> bool {
    !old.is_infinite()
        && ((new - old).abs() < rel * (new.abs() + old.abs()) * 0.5 || (rel > 0.0 && new == old))
}

impl Stopping {
    pub(super) fn ftol(&self, f: f64, old: f64) -> bool {
        relstop(old, f, self.ftol_rel)
    }

    /// `nlopt_stop_x` with zero x tolerances.
    pub(super) fn x(&self, x: &[f64], old: &[f64]) -> bool {
        x.iter().zip(old).all(|(new, old)| relstop(*old, *new, 0.0))
    }

    pub(super) fn evals(&self) -> bool {
        self.maxeval > 0 && self.nevals >= self.maxeval
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Outcome {
    Failure,
    MaxEval,
    FtolReached,
    XtolReached,
}

/// `NloptOptimizer::optimize` (`nlopt_boilerplate.hpp`) for `LN_SBPLX`: minimizes `f` from
/// `init` inside the `(lb, ub)` bounds with `ftol_rel` and at most `maxeval` evaluations.
/// Returns the optimum and its score.
pub(super) fn subplex_minimize(
    f: &mut dyn FnMut(&[f64]) -> f64,
    init: &[f64],
    (lb, ub): (&[f64], &[f64]),
    ftol_rel: f64,
    maxeval: usize,
) -> (Vec<f64>, f64) {
    let mut x = init.to_vec();
    let step = default_initial_step(&x, lb, ub);
    let mut stop = Stopping {
        ftol_rel,
        maxeval,
        nevals: 0,
    };
    let mut minf = f64::INFINITY;
    sbplx_minimize(
        &mut Problem {
            f,
            bounds: (lb, ub),
            x: &mut x,
            minf: &mut minf,
            stop: &mut stop,
        },
        &step,
    );
    (x, minf)
}

/// `nlopt_set_default_initial_step`: a quarter of the bounds, or 3/4 of the distance to a
/// closer bound.
fn default_initial_step(x: &[f64], lb: &[f64], ub: &[f64]) -> Vec<f64> {
    (0..x.len())
        .map(|i| {
            let mut step = f64::INFINITY;
            if !ub[i].is_infinite()
                && !lb[i].is_infinite()
                && (ub[i] - lb[i]) * 0.25 < step
                && ub[i] > lb[i]
            {
                step = (ub[i] - lb[i]) * 0.25;
            }
            if !ub[i].is_infinite() && ub[i] - x[i] < step && ub[i] > x[i] {
                step = (ub[i] - x[i]) * 0.75;
            }
            if !lb[i].is_infinite() && x[i] - lb[i] < step && x[i] > lb[i] {
                step = (x[i] - lb[i]) * 0.75;
            }
            if step.is_infinite() {
                if !ub[i].is_infinite() && (ub[i] - x[i]).abs() < step.abs() {
                    step = (ub[i] - x[i]) * 1.1;
                }
                if !lb[i].is_infinite() && (x[i] - lb[i]).abs() < step.abs() {
                    step = (x[i] - lb[i]) * 1.1;
                }
            }
            // `nlopt_istiny`: below the smallest normal double.
            if step.is_infinite() || (step != 0.0 && step.abs() < f64::MIN_POSITIVE) {
                step = x[i];
            }
            if step.is_infinite() || step == 0.0 {
                1.0
            } else {
                step
            }
        })
        .collect()
}

#[path = "nldrmd.rs"]
mod nldrmd;

use nldrmd::Problem;

/// Subplex strategy constants (`sbplx.c`).
const PSI: f64 = 0.25;
const OMEGA: f64 = 0.1;
const NSMIN: usize = 2;
const NSMAX: usize = 5;

/// `sbplx_minimize` (`sbplx.c:66-239`): Nelder-Mead on subspaces of the coordinates sorted by
/// decreasing progress.
fn sbplx_minimize(problem: &mut Problem, xstep0: &[f64]) -> Outcome {
    let n = problem.x.len();
    *problem.minf = (problem.f)(problem.x);
    problem.stop.nevals += 1;
    if problem.stop.evals() {
        return Outcome::MaxEval;
    }
    let mut xstep = xstep0.to_vec();
    let mut dx = vec![0.0_f64; n];
    loop {
        let xprev = problem.x.to_vec();
        let mut normi = 0.0_f64;
        let mut nsubs = 0;
        let mut fdiff_max = 0.0_f64;
        // Indices by decreasing |dx|; the glibc qsort_r merge sort is stable.
        let mut p: Vec<usize> = (0..n).collect();
        p.sort_by(|&i, &j| dx[j].abs().total_cmp(&dx[i].abs()));
        let normdx: f64 = dx.iter().map(|value| value.abs()).sum();
        let mut i = 0;
        while i + NSMIN < n {
            let ns = subspace_size(&dx, &p, i, normi, normdx);
            normi = p[i..i + ns].iter().fold(normi, |sum, &k| sum + dx[k].abs());
            nsubs += 1;
            let (outcome, fdiff) = subspace(problem, &p[i..i + ns], &xstep);
            fdiff_max = fdiff_max.max(fdiff);
            if let Some(done) = finished(outcome) {
                return done;
            }
            i += ns;
        }
        nsubs += 1;
        let (outcome, fdiff) = subspace(problem, &p[i..], &xstep);
        fdiff_max = fdiff_max.max(fdiff);
        if let Some(done) = finished(outcome) {
            return done;
        }
        if problem.stop.ftol(*problem.minf, *problem.minf + fdiff_max) {
            return Outcome::FtolReached;
        }
        // With zero x tolerances the step test of sbplx.c:180-190 passes only on zero steps.
        if problem.stop.x(problem.x, &xprev) && xstep.iter().all(|step| step.abs() * PSI <= 0.0) {
            return Outcome::XtolReached;
        }
        for (change, (new, old)) in dx.iter_mut().zip(problem.x.iter().zip(&xprev)) {
            *change = new - old;
        }
        let scale = if nsubs == 1 {
            PSI
        } else {
            let stepnorm: f64 = xstep.iter().map(|step| step.abs()).sum();
            let dxnorm: f64 = dx.iter().map(|value| value.abs()).sum();
            (dxnorm / stepnorm).clamp(OMEGA, 1.0 / OMEGA)
        };
        for (step, change) in xstep.iter_mut().zip(&dx) {
            *step = if *change == 0.0 {
                -(*step * scale)
            } else {
                (*step * scale).copysign(*change)
            };
        }
    }
}

/// The subspace length from index `i` with the sharpest drop in average |dx| (Rowan's figure of
/// merit), leaving room to partition the rest (`sbplx.c:122-145`).
fn subspace_size(dx: &[f64], p: &[usize], i: usize, normi: f64, normdx: f64) -> usize {
    let n = dx.len();
    let mut norm = p[i..i + NSMIN - 1]
        .iter()
        .fold(normi, |sum, &k| sum + dx[k].abs());
    let mut ns = NSMIN;
    let mut ns_goodness = f64::NEG_INFINITY;
    for k in i + NSMIN - 1..(i + NSMAX).min(n) {
        norm += dx[p[k]].abs();
        if n - (k + 1) < NSMIN {
            continue;
        }
        let goodness = if k + 1 < n {
            norm / (k + 1) as f64 - (normdx - norm) / (n - (k + 1)) as f64
        } else {
            normdx / n as f64
        };
        if goodness > ns_goodness {
            ns_goodness = goodness;
            ns = (k + 1) - i;
        }
    }
    ns
}

/// A subspace search that did not end on its diameter test ends subplex; a failed one counts
/// as converged.
fn finished(outcome: Outcome) -> Option<Outcome> {
    match outcome {
        Outcome::Failure => Some(Outcome::XtolReached),
        Outcome::XtolReached => None,
        other => Some(other),
    }
}

/// One subspace's Nelder-Mead on the coordinates `p` (`subspace_func`, `sbplx.c:52-63`).
fn subspace(problem: &mut Problem, p: &[usize], xstep: &[f64]) -> (Outcome, f64) {
    let (lb, ub) = problem.bounds;
    let mut xs: Vec<f64> = p.iter().map(|&k| problem.x[k]).collect();
    let steps: Vec<f64> = p.iter().map(|&k| xstep[k]).collect();
    let lbs: Vec<f64> = p.iter().map(|&k| lb[k]).collect();
    let ubs: Vec<f64> = p.iter().map(|&k| ub[k]).collect();
    let base = problem.x.to_vec();
    let f = &mut *problem.f;
    let mut sub = |xs: &[f64]| {
        let mut full = base.clone();
        for (value, &k) in xs.iter().zip(p) {
            full[k] = *value;
        }
        f(&full)
    };
    let result = Problem {
        f: &mut sub,
        bounds: (&lbs, &ubs),
        x: &mut xs,
        minf: &mut *problem.minf,
        stop: &mut *problem.stop,
    }
    .minimize(&steps, PSI);
    for (value, &k) in xs.iter().zip(p) {
        problem.x[k] = *value;
    }
    result
}
