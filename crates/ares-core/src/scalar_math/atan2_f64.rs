//! CORE-MATH binary64 `atan2`, revision `3270006242ceef2885767918533ac5446807f2cf`,
//! `src/binary64/atan2/atan2.c`.
/* Correctly-rounded atan2 function for two binary64 values.

Copyright (c) 2024-2025 Paul Zimmermann and Alexei Sibidanov

This file is part of the CORE-MATH project
(https://core-math.gitlabpages.inria.fr/).

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

mod rational_p;
mod rational_q;
mod tables;
mod tint;

use rational_p::P;
use rational_q::Q;
use std::cmp::Ordering;
use tables::{B, B2, F2, FINF, O, PI_H, PI_L, PI_OVER2_H, PI_OVER2_L, T2};
use tint::{PI, PI2, Tint};

const ABS_MASK: u64 = 0x7fff_ffff_ffff_ffff;
const INF_SHIFTED: u64 = 0x7ff_u64 << 53;

#[inline]
fn fast_two_sum(x: f64, y: f64) -> (f64, f64) {
    let s = x + y;
    let z = s - x;
    (s, y - z)
}

#[inline]
fn fast_sum(xh: f64, xl: f64, yh: f64, yl: f64) -> (f64, f64) {
    let (sh, sl) = fast_two_sum(xh, yh);
    (sh, (xl + yl) + sl)
}

/// Decompose positive finite `v` into exact `(mantissa, exp2)` with `v = mantissa * 2^exp2`
/// and `2^52 <= mantissa < 2^53`.
#[inline]
fn decompose_positive(v: f64) -> (u64, i32) {
    let bits = v.to_bits();
    let exp = (bits >> 52) as i32;
    if exp != 0 {
        ((1_u64 << 52) | (bits & 0x000f_ffff_ffff_ffff), exp - 1075)
    } else {
        let lz = bits.leading_zeros() as i32;
        let shift = lz - 11;
        (bits << shift, -1074 - shift)
    }
}

/// Exact round-to-nearest-even `atan2(y, x)` for `x > 0` and tiny `0 < |y / x| < 2^-30`
/// (including subnormal ratios and 151-bit cancellation pairs).
fn atan2_tiny_positive_x(y: f64, x: f64, t: f64) -> f64 {
    let a = y.abs();
    let t_abs = t.abs();
    let t_bits = t_abs.to_bits();
    let t_prev = f64::from_bits(t_bits - 1);

    // Exact midpoint `m_lo = (t_prev + t_abs) / 2 = m_mant * 2^m_exp`.
    let t_exp = (t_bits >> 52) as i32;
    let t_frac = t_bits & 0x000f_ffff_ffff_ffff;
    let (m_mant, m_exp): (u64, i32) = if t_exp == 0 {
        (2 * t_frac - 1, -1075)
    } else if t_frac == 0 {
        if t_exp == 1 {
            ((1_u64 << 53) - 1, -1075)
        } else {
            ((1_u64 << 54) - 1, t_exp - 1077)
        }
    } else {
        ((1_u64 << 53) + 2 * t_frac - 1, t_exp - 1076)
    };

    let (ma, ea) = decompose_positive(a);
    let (mb, eb) = decompose_positive(x);
    let prod = u128::from(m_mant) * u128::from(mb);
    let ep = m_exp + eb;
    let shift = ea - ep;

    // Compare `a` against `m_lo * x` exactly in 128-bit integer arithmetic.
    let (lhs, rhs, e_common) = if shift >= 0 {
        if shift >= 75 {
            return t;
        }
        (u128::from(ma) << (shift as u32), prod, ep)
    } else {
        let neg = (-shift) as u32;
        if neg >= 21 {
            return t_prev.copysign(y);
        }
        (u128::from(ma), prod << neg, ea)
    };

    if lhs <= rhs {
        // `|y| / x <= m_lo`, so `|atan(y / x)| < |y| / x <= m_lo`.
        return t_prev.copysign(y);
    }

    // Exact positive gap `(|y| - m_lo * x) / x`, evaluated to 185+ bits past cancellation.
    let gap = Tint::from_u128_exp2(lhs - rhs, i64::from(e_common)).div(Tint::from_f64(x));

    // `r - atan(r) = r^3/3 - r^5/5 + r^7/7 - ...`; for `r < 2^-30`, 3 terms have < 2^-180 rel err.
    let r = Tint::div_f64(a, x);
    let r2 = r.mul(r);
    let r3 = r.mul(r2);
    let r5 = r3.mul(r2);
    let r7 = r5.mul(r2);
    const INV3: Tint = Tint::new(
        0xaaaa_aaaa_aaaa_aaaa,
        0xaaaa_aaaa_aaaa_aaaa,
        0xaaaa_aaaa_aaaa_aaab,
        -1,
        0,
    );
    const INV5: Tint = Tint::new(
        0xcccc_cccc_cccc_cccc,
        0xcccc_cccc_cccc_cccc,
        0xcccc_cccc_cccc_cccd,
        -2,
        1,
    );
    const INV7: Tint = Tint::new(
        0x9249_2492_4924_9249,
        0x2492_4924_9249_2492,
        0x4924_9249_2492_4925,
        -2,
        0,
    );
    let drop = r3.mul(INV3).add(r5.mul(INV5)).add(r7.mul(INV7));

    if gap.cmp_abs(drop) == Ordering::Greater {
        t
    } else {
        t_prev.copysign(y)
    }
}

#[cold]
#[inline(never)]
fn atan2_accurate(y: f64, x: f64) -> f64 {
    let t = y / x;
    if t == 0.0 {
        if x > 0.0 {
            return t;
        }
        return if y > 0.0 { PI_H + PI_L } else { -PI_H - PI_L };
    }

    let corr = t.mul_add(x, -y);
    if corr == 0.0 && x > 0.0 && t.abs() <= f64::from_bits(0x3e4d_12ed_0af1_a27f) {
        if y.abs() >= f64::from_bits(0x06c0_0000_0000_0000) {
            return t;
        }
        let scale = f64::from_bits(0x4680_0000_0000_0000);
        if (t * scale).mul_add(x, -(y * scale)) == 0.0 {
            return t;
        }
    }

    if x > 0.0 && t.abs() < f64::from_bits(0x3e10_0000_0000_0000) {
        return atan2_tiny_positive_x(y, x, t);
    }

    let inv = y.abs() > x.abs();
    let mut z = if inv {
        Tint::div_f64(x, y)
    } else {
        Tint::div_f64(y, x)
    };

    let sz = z.sgn;
    z.sgn = 0;
    let mut p = P[29];
    let mut q = Q[29];
    for i in (0..29).rev() {
        p = p.mul(z).add(P[i]);
        q = q.mul(z).add(Q[i]);
    }
    p = p.mul(z);
    z = p.div(q);
    z.sgn = sz;

    if inv {
        if z.sgn == 0 {
            z.sgn = 1;
            z = PI2.add(z);
        } else {
            z = PI2.add(z);
            z.sgn = 1;
        }
    }
    if x < 0.0 {
        if z.sgn == 0 {
            z.sgn = 1;
            z = PI.add(z);
            z.sgn = 1;
        } else {
            z = PI.add(z);
        }
    }
    z.to_f64(false)
}

#[cold]
#[inline(never)]
fn atan2_special(y0: f64, x0: f64) -> f64 {
    let iy = y0.to_bits();
    let ix = x0.to_bits();
    let aiy = iy << 1;
    let aix = ix << 1;

    if aiy >= INF_SHIFTED || aix >= INF_SHIFTED {
        if aiy > INF_SHIFTED || aix > INF_SHIFTED {
            return y0 + x0;
        }
        if aiy == INF_SHIFTED && aix == INF_SHIFTED {
            let idx = (ix >> 63) as usize;
            return FINF[idx][1].copysign(y0) + FINF[idx][0].copysign(y0);
        }
        if aix == INF_SHIFTED {
            if x0 < 0.0 {
                return PI_H.copysign(y0) + PI_L.copysign(y0);
            }
            return 0.0_f64.copysign(y0);
        }
        return PI_OVER2_H.copysign(y0) + PI_OVER2_L.copysign(y0);
    }

    if aiy == 0 && aix == 0 {
        if ix == 0 {
            return y0;
        }
        return if iy == 0 { PI_H + PI_L } else { -PI_H - PI_L };
    }
    if aiy == 0 {
        if x0 > 0.0 {
            return y0 / x0;
        }
        return if (iy >> 63) == 0 {
            PI_H + PI_L
        } else {
            -PI_H - PI_L
        };
    }
    if y0 > 0.0 {
        PI_OVER2_H + PI_OVER2_L
    } else {
        -PI_OVER2_H - PI_OVER2_L
    }
}

/// Correctly-rounded IEEE-754 binary64 `atan2(y, x)` (round-to-nearest-even).
pub(crate) fn atan2_f64(y0: f64, x0: f64) -> f64 {
    let iy = y0.to_bits();
    let ix = x0.to_bits();
    let aiy = iy & ABS_MASK;
    if aiy == 0 || aiy >= (0x7ff_u64 << 52) {
        return atan2_special(y0, x0);
    }
    let aix = ix & ABS_MASK;
    if aix == 0 || aix >= (0x7ff_u64 << 52) {
        return atan2_special(y0, x0);
    }

    let ax = x0.abs();
    let ay = y0.abs();
    let (mut x, mut y) = if ax >= ay { (ax, ay) } else { (ay, ax) };
    let sy = iy >> 63;
    let sx = ix >> 63;
    let gt = u64::from(aix < aiy);
    let dxy = aix.wrapping_sub(aiy) ^ gt.wrapping_neg();
    if dxy >= (53_u64 << 52) {
        return atan2_accurate(y0, x0);
    }

    let sgn = if ((gt ^ sx ^ sy) & 1) != 0 { -1.0 } else { 1.0 };
    let kw = ((sx << 2) | (sy << 1) | gt) as usize;
    let jj = (y / x + (2.0 + 1.0 / 128.0)).to_bits();
    let jt = ((jj >> (52 - 7)) & 127) as usize;
    let mut fh = F2[jt][1] * sgn + O[kw][0];
    let mut fl = F2[jt][0] * sgn + O[kw][1];

    if x < f64::from_bits(0x0670_0000_0000_0000) {
        let scale = f64::from_bits(0x7970_0000_0000_0000);
        x *= scale;
        y *= scale;
    }
    if x > f64::from_bits(0x7fd0_0000_0000_0000) && jt != 0 {
        x *= 0.5;
        y *= 0.5;
    }

    let t0 = T2[jt];
    let zn = (-t0).mul_add(x, y);
    let zd = t0.mul_add(y, x);
    let mut z = zn / zd;
    let mut z2 = z * z;
    z *= sgn;
    let dz = (z * z2) * (B[0] + z2 * (B[1] + z2 * B[2]));
    let eps =
        z.abs() * f64::from_bits(0x3cc0_5100_0000_0000) + f64::from_bits(0x3a50_0000_0000_0000);
    let (rh, z_err) = fast_two_sum(fh, z);
    let rl = (fl + dz) + z_err;
    let lb = rh + (rl - eps);
    let ub = rh + (rl + eps);
    if lb != ub {
        let dh0 = y * t0;
        let dl0 = y.mul_add(t0, -dh0);
        let (dh, e) = fast_two_sum(x, dh0);
        let rdh = 1.0 / dh;
        let dl = dl0 + e;
        let nh0 = x * t0;
        let nl0 = x.mul_add(t0, -nh0);
        let dt = y - nh0;
        let y1 = dt + nh0;
        let (nh, nl) = if y1 == y {
            fast_two_sum(dt, -nl0)
        } else {
            fast_two_sum(dt, (y - y1) - nl0)
        };
        let mut zh = nh * rdh;
        z2 = zh * zh;
        let mut zl = rdh * (dh.mul_add(-zh, nh) + (nl - (nh * rdh) * dl));
        zl += zh * z2 * ((B2[0] + z2 * B2[1]) + (z2 * z2) * (B2[2] + z2 * B2[3]));
        zh *= sgn;
        zl *= sgn;
        let eps2 = f64::from_bits(0x3cd4_0000_0000_0000)
            * (zh.abs() * z2 + f64::from_bits(0x3cc0_0000_0000_0000));
        (fh, fl) = fast_sum(fh, fl, zh, zl);
        let lb2 = fh + (fl - eps2);
        let ub2 = fh + (fl + eps2);
        if lb2 != ub2 {
            return atan2_accurate(y0, x0);
        }
        return ub2;
    }
    ub
}
