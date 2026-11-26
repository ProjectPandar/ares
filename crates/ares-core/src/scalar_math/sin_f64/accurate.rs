//! CORE-MATH binary64 sin accurate paths, revision 3270006242ceef2885767918533ac5446807f2cf, src/binary64/sin/sin.c.
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

use super::{
    fast_two_sum, mul_dd,
    tables_accurate::{C2U, PC, PS, S1U, S2U},
    tables_fast::T,
};

#[inline]
fn mh_uu(a: u128, b: u128) -> u128 {
    let (ah, al) = ((a >> 64) as u64, a as u64);
    let (bh, bl) = ((b >> 64) as u64, b as u64);
    let ahbh = u128::from(ah) * u128::from(bh);
    let ahbl = u128::from(ah) * u128::from(bl);
    let albh = u128::from(al) * u128::from(bh);
    ahbh + (ahbl >> 64) + (albh >> 64)
}

#[inline]
fn eval_ps(u: u128, u2: u128, u2h: u128, u4: u128) -> u128 {
    let sh = mh_uu(PS[2] - PS[3] * u2h, u4);
    let s = PS[0] - mh_uu(PS[1], u2) + sh;
    mh_uu(s, u)
}

#[inline]
fn eval_pc(u2: u128, u2h: u128, u4: u128) -> u128 {
    let sh = PC[2] - mh_uu(PC[3] - u2h * PC[4], u2);
    (PC[0] - mh_uu(PC[1], u2)) + mh_uu(sh, u4)
}

/// Round `(-1)^s * r / 2^128` to binary64, assuming `r >> 64 != 0`.
#[inline]
fn u128_to_f64(r: u128, s: usize) -> f64 {
    const SGN: [f64; 2] = [
        f64::from_bits(0x3ca0_0000_0000_0000), // 2^-53
        f64::from_bits(0xbca0_0000_0000_0000), // -2^-53
    ];
    const LOW: [f64; 2] = [
        f64::from_bits(0x3fd8_0000_0000_0000), // 0x1.8p-2
        f64::from_bits(0x3fe8_0000_0000_0000), // 0x1.8p-1
    ];
    let sh = ((r >> 64) as u64).leading_zeros();
    let h = (r >> (75 - sh)) as u64;
    let rbit = ((r >> (74 - sh)) & 1) as usize;
    let v = f64::from_bits(SGN[s].to_bits() - (u64::from(sh) << 52));
    let a = (h as f64) * v;
    let b = LOW[rbit] * v;
    a + b
}

/// 128-bit Payne-Hanek reduction for `|x| >= 2^-16`.
fn reduce_large_acc(x: f64) -> (u64, u128, bool) {
    const MASK: u128 = (1u128 << 115) - 1;
    let bits = x.to_bits();
    let e = ((bits >> 52) & 0x7ff) as usize;
    let m = (1u64 << 52) | (bits & 0x000f_ffff_ffff_ffff);
    let i_plus_1 = (e - 947) / 64;
    let f = (e.wrapping_sub(1011) & 0x3f) as u32;
    let mut v0 = if i_plus_1 > 0 { T[i_plus_1 - 1] } else { 0 };
    let mut v1 = T[i_plus_1];
    let mut v2 = T[i_plus_1 + 1];
    if f != 0 {
        v0 = (v0 << f) | (v1 >> (64 - f));
        v1 = (v1 << f) | (v2 >> (64 - f));
        v2 = (v2 << f) | (T[i_plus_1 + 2] >> (64 - f));
    }
    let mut u = (u128::from(v1) | (u128::from(v0) << 64)).wrapping_mul(u128::from(m));
    let v = u128::from(m) * u128::from(v2);
    u = u.wrapping_add(v >> 64);
    let mut k = (u >> (128 - 13)) as u64;
    u &= MASK;
    let neg = (u >> 114) != 0;
    if neg {
        k = (k + 1) & ((1u64 << 13) - 1);
        u = (MASK + 1) - u;
    }
    (k, u, neg)
}

/// Accurate path for `|x| < 2^-16`.
#[cold]
#[inline(never)]
pub(super) fn sin_small_accurate(x: f64) -> f64 {
    const C3H: f64 = f64::from_bits(0xbfc5_5555_5555_5555); // -0x1.5555555555555p-3
    const C3L: f64 = f64::from_bits(0xbc65_5554_b00d_e7e8); // -0x1.55554b00de7e8p-57
    const C5: f64 = f64::from_bits(0x3f81_1111_1110_8480); // 0x1.111111110848p-7

    let x2h = x * x;
    let x2l = x.mul_add(x, -x2h);
    let h = C5 * x2h + C3L;
    let (h, l) = fast_two_sum(C3H, h);
    let (h, l) = mul_dd(h, l, x2h, x2l);
    let (h, l) = mul_dd(h, l, x, 0.0);
    let (h, t) = fast_two_sum(x, h);
    h + (l + t)
}

/// Accurate path for `|x| >= 2^-16`.
#[cold]
#[inline(never)]
pub(super) fn sin_large_accurate(x: f64) -> f64 {
    let (k, r, neg) = reduce_large_acc(x);
    let r = r << 3;
    let u2 = mh_uu(r, r);
    let u4 = mh_uu(u2, u2);
    let u2h = u2 >> 64;

    let mut sbit = usize::from(x.is_sign_negative()) ^ (((k >> 12) & 1) as usize);
    let i1 = ((k >> 6) & 0x3f) as usize;
    let i2 = (k & 0x3f) as usize;

    let s1 = if i1 <= 32 { S1U[i1] } else { S1U[64 - i1] };
    let c1 = if i1 <= 32 { S1U[32 - i1] } else { S1U[i1 - 32] };

    let mut s1u = mh_uu(s1, C2U[i2]);
    let t = mh_uu(c1, S2U[i2]);
    s1u = if i1 < 32 { s1u + t } else { s1u - t };

    let mut c1u = mh_uu(c1, C2U[i2]);
    let t = mh_uu(s1, S2U[i2]);
    c1u = if i1 < 32 { c1u - t } else { c1u + t };

    let sr = eval_ps(r, u2, u2h, u4);
    let cr = eval_pc(u2, u2h, u4);

    s1u = mh_uu(s1u, cr);
    c1u = mh_uu(c1u, sr);

    if (i1 < 32) ^ neg {
        s1u += c1u;
    } else if s1u < c1u {
        s1u = c1u - s1u;
        sbit ^= 1;
    } else {
        s1u -= c1u;
    }
    u128_to_f64(s1u, sbit)
}
