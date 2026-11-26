//! CORE-MATH binary64 sin, revision 3270006242ceef2885767918533ac5446807f2cf, src/binary64/sin/sin.c.
/* Correctly-rounded sine function for binary64 value.

Copyright (c) 2026 Paul Zimmermann and Alexei Sibidanov

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

mod accurate;
mod tables_accurate;
mod tables_fast;
mod tables_u2;

use tables_fast::{T, U1_SIN};
use tables_u2::{U2_COS, U2_SIN};

const PIH: f64 = f64::from_bits(0xbf29_21fb_5444_2d18); // -0x1.921fb54442d18p-13
const PIL: f64 = f64::from_bits(0xbbc1_a626_3314_5c07); // -0x1.1a62633145c07p-67

/// Correctly rounded IEEE-754 binary64 sine (round-to-nearest-even, `<= 0.5` ULP).
pub(crate) fn sin_f64(x: f64) -> f64 {
    let bits = x.to_bits();
    let e = ((bits >> 52) & 0x7ff) as u32;
    if e < 0x3ff - 26 {
        // |x| < 2^-26: |sin(x) - x| < 1/2 ulp
        let au = bits << 1;
        if au == 0 {
            return x;
        }
        return x.mul_add(f64::from_bits(0xbc90_0000_0000_0000), x);
    }
    if e < 0x3ff + 31 {
        return sin_moderate(x, (bits >> 63) as usize);
    }
    if e == 0x7ff {
        return f64::from_bits(0x7ff8_0000_0000_0000);
    }
    sin_large(x)
}

#[inline]
pub(super) fn fast_two_sum(x: f64, y: f64) -> (f64, f64) {
    let s = x + y;
    let z = s - x;
    (s, y - z)
}

#[inline]
fn fast_sum(xh: f64, xl: f64, yh: f64, yl: f64) -> (f64, f64) {
    let (sh, sl) = fast_two_sum(xh, yh);
    (sh, (xl + yl) + sl)
}

#[inline]
pub(super) fn mul_dd(xh: f64, xl: f64, ch: f64, cl: f64) -> (f64, f64) {
    let ahhh = xh * ch;
    let l = (xh * cl + xl * ch) + xh.mul_add(ch, -ahhh);
    (ahhh, l)
}

#[inline]
fn u1_sincos(i1: usize) -> (f64, f64, f64, f64) {
    let (si, ci, c_sign) = if i1 <= 64 {
        (i1, 64 - i1, 0u64)
    } else {
        (128 - i1, i1 - 64, 1u64 << 63)
    };
    let [sh, sl] = U1_SIN[si];
    let [ch, cl] = U1_SIN[ci];
    (
        f64::from_bits(sh),
        f64::from_bits(sl),
        f64::from_bits(ch ^ c_sign),
        f64::from_bits(cl ^ c_sign),
    )
}

#[inline]
fn combine_tables(i1: usize, i2: usize) -> (f64, f64, f64) {
    let (u1_sh, u1_sl, u1_ch, u1_cl) = u1_sincos(i1);
    let [u2_sh_b, u2_sl_b] = U2_SIN[i2];
    let [u2_ch_b, u2_cl_b] = U2_COS[i2];
    let u2_sh = f64::from_bits(u2_sh_b);
    let u2_sl = f64::from_bits(u2_sl_b);
    let u2_ch = f64::from_bits(u2_ch_b);
    let u2_cl = f64::from_bits(u2_cl_b);
    let (s1h, s1l) = mul_dd(u1_sh, u1_sl, u2_ch, u2_cl);
    let (s2h, s2l) = mul_dd(u2_sh, u2_sl, u1_ch, u1_cl);
    let (sh, sl) = fast_sum(s1h, s1l, s2h, s2l);
    let ch = u1_ch * u2_ch - u1_sh * u2_sh;
    (sh, sl, ch)
}

/// Fast path for `2^-26 <= |x| < 2^31`.
#[inline]
fn sin_moderate(x: f64, mut sbit: usize) -> f64 {
    const INV_PI: f64 = f64::from_bits(0x40b4_5f30_6dc9_c883); // 0x1.45f306dc9c883p+12
    const MOD_S3: f64 = f64::from_bits(0x3fc5_5555_5530_68f0); // 0x1.55555553068fp-3
    const MOD_C4: f64 = f64::from_bits(0x3fa5_5555_553b_fd30); // 0x1.55555553bfd3p-5
    const EPS: f64 = f64::from_bits(0x3bfd_e000_0000_0000); // 0x1.dep-64
    const SGN: [f64; 2] = [1.0, -1.0];

    let ax = x.abs();
    let k = (INV_PI * ax).round_ties_even();
    let rh = k.mul_add(PIH, ax);
    let rl = k * PIL;
    let r = rh + rl;
    let r2 = r * r;
    let j = k as i64;
    sbit ^= ((j >> 14) & 1) as usize;
    let i1 = ((j >> 7) & 0x7f) as usize;
    let i2 = (j & 0x7f) as usize;

    let (tab_sh, tab_sl, tab_ch) = combine_tables(i1, i2);
    let sh = r * (1.0 - MOD_S3 * r2);
    let ch = r2 * (-0.5 + MOD_C4 * r2);
    let fh = SGN[sbit] * tab_sh;
    let fl = SGN[sbit] * ((tab_sl + tab_sh * ch) + tab_ch * sh);
    let lb = fh + (fl - EPS);
    let ub = fh + (fl + EPS);
    if ub == lb {
        return lb;
    }
    if ax < f64::from_bits(0x3ef0_0000_0000_0000) {
        return accurate::sin_small_accurate(x);
    }
    accurate::sin_large_accurate(x)
}

/// Argument reduction for `x >= 2^31` (`x > 0` finite).
fn reduce_large(x: f64) -> (u64, f64) {
    const MAGIC: u128 = (1u128 << 112) + (129u128 << 52);
    const SCALE_R: f64 = f64::from_bits(0x3bb0_0000_0000_0000); // 2^-68
    const BIAS_R: f64 = f64::from_bits(0x3ef0_0000_0000_0000); // 2^-16

    let bits = x.to_bits();
    let e = ((bits >> 52) & 0x7ff) as usize;
    let m = (1u64 << 52) | (bits & 0x000f_ffff_ffff_ffff);
    let i = (e - 1011) / 64;
    let f = ((e - 1011) & 0x3f) as u32;
    let (v0, v1) = if f == 0 {
        (T[i], T[i + 1])
    } else {
        (
            (T[i] << f) | (T[i + 1] >> (64 - f)),
            (T[i + 1] << f) | (T[i + 2] >> (64 - f)),
        )
    };
    let u = (u128::from(v1) | (u128::from(v0) << 64))
        .wrapping_mul(u128::from(m))
        .wrapping_add(MAGIC);
    let r_int = ((u << 15) >> 75) as u64;
    let r = (r_int as f64) * SCALE_R - BIAS_R;
    ((u >> 113) as u64, r)
}

/// Fast path for `|x| >= 2^31`.
#[inline(never)]
fn sin_large(x: f64) -> f64 {
    const LRG_S1: f64 = f64::from_bits(0x4019_21fb_5444_2d18); // 0x1.921fb54442d18p2
    const LRG_S3: f64 = f64::from_bits(0x4044_abbc_db6b_26d1); // 0x1.4abbcdb6b26d1p5
    const LRG_C2: f64 = f64::from_bits(0xc033_bd3c_c9be_45de); // -0x1.3bd3cc9be45dep4
    const LRG_C4: f64 = f64::from_bits(0x4050_3c1e_ee48_3083); // 0x1.03c1eee483083p6
    const EPS: f64 = f64::from_bits(0x3c00_1000_0000_0000); // 0x1.01p-63
    const SGN: [f64; 2] = [1.0, -1.0];

    let (j, r) = reduce_large(x.abs());
    let sbit = usize::from(x.is_sign_negative()) ^ ((j >> 14) as usize);
    let i1 = ((j >> 7) & 0x7f) as usize;
    let i2 = (j & 0x7f) as usize;

    let (tab_sh, tab_sl, tab_ch) = combine_tables(i1, i2);
    let r2 = r * r;
    let sh = r * (LRG_S1 - LRG_S3 * r2);
    let ch = r2 * (LRG_C2 + LRG_C4 * r2);
    let fh = SGN[sbit] * tab_sh;
    let fl = SGN[sbit] * ((tab_sl + tab_sh * ch) + tab_ch * sh);
    let lb = fh + (fl - EPS);
    let ub = fh + (fl + EPS);
    if lb == ub {
        return lb;
    }
    accurate::sin_large_accurate(x)
}
