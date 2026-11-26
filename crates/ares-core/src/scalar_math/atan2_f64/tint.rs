//! CORE-MATH 192-bit arithmetic routines, revision `3270006242ceef2885767918533ac5446807f2cf`,
//! `src/binary64/atan2/tint.h`.
/* Fast 192-bit arithmetic routines.

Copyright (c) 2023-2024 Paul Zimmermann and Alexei Sibidanov

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

use std::cmp::Ordering;

/// Normalized 192-bit binary floating-point value:
/// `(-1)^sgn * (hm / 2^128 + l / 2^192) * 2^ex`, with either `hm == 0 && l == 0`
/// (zero) or bit 127 of `hm` set.
#[derive(Clone, Copy)]
pub(super) struct Tint {
    pub(super) hm: u128,
    pub(super) l: u64,
    pub(super) ex: i64,
    pub(super) sgn: u64,
}

pub(super) const ZERO: Tint = Tint::new(0, 0, 0, -1076, 0);
pub(super) const ONE: Tint = Tint::new(0x8000_0000_0000_0000, 0, 0, 1, 0);
pub(super) const PI: Tint = Tint::new(
    0xc90f_daa2_2168_c234,
    0xc4c6_628b_80dc_1cd1,
    0x2902_4e08_8a67_cc74,
    2,
    0,
);
pub(super) const PI2: Tint = Tint::new(
    0xc90f_daa2_2168_c234,
    0xc4c6_628b_80dc_1cd1,
    0x2902_4e08_8a67_cc74,
    1,
    0,
);

impl Tint {
    pub(super) const fn new(h: u64, m: u64, l: u64, ex: i64, sgn: u64) -> Self {
        Self {
            hm: ((h as u128) << 64) | (m as u128),
            l,
            ex,
            sgn,
        }
    }

    #[inline]
    pub(super) const fn h(self) -> u64 {
        (self.hm >> 64) as u64
    }

    #[inline]
    pub(super) const fn m(self) -> u64 {
        self.hm as u64
    }

    #[inline]
    pub(super) const fn is_zero(self) -> bool {
        self.h() == 0
    }

    /// Exact conversion from a finite nonzero `f64`.
    #[inline]
    pub(super) fn from_f64(x: f64) -> Self {
        let u = x.to_bits();
        let sgn = u >> 63;
        let ax = u & 0x7fff_ffff_ffff_ffff;
        let e = ax >> 52;
        let (h, ex) = if e != 0 {
            ((1_u64 << 63) | (ax << 11), e as i64 - 0x3fe)
        } else {
            let lz = ax.leading_zeros() as i64;
            (ax << lz, -0x3f2 - lz)
        };
        Self::new(h, 0, 0, ex, sgn)
    }

    /// Exact conversion from a positive `u128` value `val * 2^exp2`.
    #[inline]
    pub(super) fn from_u128_exp2(val: u128, exp2: i64) -> Self {
        let lz = val.leading_zeros();
        Self {
            hm: val << lz,
            l: 0,
            ex: exp2 + 128 - i64::from(lz),
            sgn: 0,
        }
    }

    #[inline]
    fn pow2(i: i64) -> f64 {
        f64::from_bits(((i + 1023) as u64) << 52)
    }

    /// Convert to correctly-rounded binary64 under round-to-nearest-even.
    #[inline]
    pub(super) fn to_f64(self, exact_truncate: bool) -> f64 {
        if self.ex >= 1025 {
            let max = f64::from_bits(0x7fe0_0000_0000_0000);
            return if self.sgn != 0 { -max - max } else { max + max };
        }
        if self.ex <= -1074 {
            let min_sub = f64::from_bits(0x0000_0000_0000_0001);
            let signed = if self.sgn != 0 { -min_sub } else { min_sub };
            let mid = self.ex == -1074 && self.hm == (1_u128 << 127) && self.l == 0;
            return signed * if self.ex < -1074 || mid { 0.5 } else { 0.75 };
        }
        let (mut hh, mut mm, mut ll, mut ex) = (self.h(), self.m(), self.l, self.ex);
        if ex <= -1022 {
            let sh = (-1021 - ex) as u32;
            ll = (mm << (64 - sh)) | (ll >> sh) | u64::from(ll > 0);
            mm = (hh << (64 - sh)) | (mm >> sh);
            hh >>= sh;
            ex += i64::from(sh);
        }
        let low = hh & 0x7ff;
        let h = (hh >> 11) as f64;
        let l: f64 = if exact_truncate {
            0.0
        } else if low < 0x400 {
            0.25
        } else if low > 0x400 || mm != 0 || ll != 0 {
            0.75
        } else {
            0.5
        };
        let s = if self.sgn != 0 { -1.0 } else { 1.0 };
        l.mul_add(s, s * h) * f64::from_bits(0x3cb0_0000_0000_0000) * Self::pow2(ex - 1)
    }

    #[inline]
    pub(super) fn cmp_abs(self, other: Self) -> Ordering {
        if self.is_zero() {
            return if other.is_zero() {
                Ordering::Equal
            } else {
                Ordering::Less
            };
        }
        if other.is_zero() {
            return Ordering::Greater;
        }
        self.ex
            .cmp(&other.ex)
            .then_with(|| self.hm.cmp(&other.hm))
            .then_with(|| self.l.cmp(&other.l))
    }

    /// Multiply two `Tint` values (error `< 10` ulps of 192 bits).
    #[inline]
    pub(super) fn mul(self, other: Self) -> Self {
        let mut ex = self.ex + other.ex;
        let sgn = self.sgn ^ other.sgn;
        let (ah, am, al) = (
            u128::from(self.h()),
            u128::from(self.m()),
            u128::from(self.l),
        );
        let (bh, bm, bl) = (
            u128::from(other.h()),
            u128::from(other.m()),
            u128::from(other.l),
        );
        let (rh, rm1, rm2) = (ah * bh, ah * bm, am * bh);
        let rl_sum = ((ah * bl) >> 64) + ((am * bm) >> 64) + ((al * bh) >> 64);

        let mut h = (rh >> 64) as u64;
        let (mut m, c1) = (rh as u64).overflowing_add((rm1 >> 64) as u64);
        h = h.wrapping_add(u64::from(c1));

        let (mut l, cl2) = (rm1 as u64).overflowing_add(rm2 as u64);
        let mut cm = u64::from(cl2);
        let (m2, c2) = m.overflowing_add((rm2 >> 64) as u64);
        m = m2;
        h = h.wrapping_add(u64::from(c2));

        cm = cm.wrapping_add((rl_sum >> 64) as u64);
        let (l3, cl3) = l.overflowing_add(rl_sum as u64);
        l = l3;
        cm = cm.wrapping_add(u64::from(cl3));

        let (m3, c3) = m.overflowing_add(cm);
        m = m3;
        h = h.wrapping_add(u64::from(c3));

        if (h >> 63) == 0 {
            h = (h << 1) | (m >> 63);
            m = (m << 1) | (l >> 63);
            l <<= 1;
            ex -= 1;
        }
        Self::new(h, m, l, ex, sgn)
    }

    #[inline]
    fn rshift_sig(hm: u128, l: u64, k: u64) -> (u128, u64) {
        match k {
            0 => (hm, l),
            1..=63 => (hm >> k, ((hm << (64 - k)) as u64) | (l >> k)),
            64 => (hm >> 64, hm as u64),
            65..=127 => (hm >> k, (hm >> (k - 64)) as u64),
            128..=191 => (0, (hm >> (k - 64)) as u64),
            _ => (0, 0),
        }
    }

    #[inline]
    fn lshift_sig(hm: u128, l: u64, k: u32) -> (u128, u64) {
        match k {
            0 => (hm, l),
            1..=63 => ((hm << k) | u128::from(l >> (64 - k)), l << k),
            64 => ((hm << 64) | u128::from(l), 0),
            65..=127 => ((hm << k) | (u128::from(l) << (k - 64)), 0),
            128..=191 => (u128::from(l) << (k - 64), 0),
            _ => (0, 0),
        }
    }

    #[inline]
    fn leading_zeros_192(hm: u128, l: u64) -> u32 {
        if hm != 0 {
            hm.leading_zeros()
        } else {
            128 + l.leading_zeros()
        }
    }

    /// Add two `Tint` values (error bounded by 2 ulps of 192 bits).
    #[inline]
    pub(super) fn add(self, other: Self) -> Self {
        let (a, b) = match self.cmp_abs(other) {
            Ordering::Equal => {
                if (self.sgn ^ other.sgn) != 0 {
                    return ZERO;
                }
                return Self {
                    ex: self.ex + 1,
                    ..self
                };
            }
            Ordering::Less => (other, self),
            Ordering::Greater => (self, other),
        };

        let sh = (a.ex - b.ex) as u64;
        let (t_hm, t_l) = Self::rshift_sig(b.hm, b.l, sh);

        if (a.sgn ^ b.sgn) != 0 {
            let (dl, borrow) = a.l.overflowing_sub(t_l);
            let dhm = a.hm.wrapping_sub(t_hm).wrapping_sub(u128::from(borrow));
            let ex = Self::leading_zeros_192(dhm, dl);
            if ex <= 1 || sh == 0 {
                let (hm, l) = Self::lshift_sig(dhm, dl, ex);
                Self {
                    hm,
                    l,
                    ex: a.ex - i64::from(ex),
                    sgn: a.sgn,
                }
            } else {
                let (b_hm, b_l) = Self::lshift_sig(b.hm, b.l, ex - sh as u32);
                let (a_hm, a_l) = Self::lshift_sig(a.hm, a.l, ex);
                let (dl2, borrow2) = a_l.overflowing_sub(b_l);
                let dhm2 = a_hm.wrapping_sub(b_hm).wrapping_sub(u128::from(borrow2));
                let ex1 = Self::leading_zeros_192(dhm2, dl2);
                let (hm, l) = Self::lshift_sig(dhm2, dl2, ex1);
                Self {
                    hm,
                    l,
                    ex: a.ex - i64::from(ex + ex1),
                    sgn: a.sgn,
                }
            }
        } else {
            let (r_l, cl) = a.l.overflowing_add(t_l);
            let (r_hm1, ch1) = a.hm.overflowing_add(t_hm);
            let (r_hm, ch2) = r_hm1.overflowing_add(u128::from(cl));
            let ch = u128::from(ch1) + u128::from(ch2);
            if ch != 0 {
                Self {
                    hm: (ch << 127) | (r_hm >> 1),
                    l: ((r_hm as u64) << 63) | (r_l >> 1),
                    ex: a.ex + 1,
                    sgn: a.sgn,
                }
            } else {
                Self {
                    hm: r_hm,
                    l: r_l,
                    ex: a.ex,
                    sgn: a.sgn,
                }
            }
        }
    }

    /// Approximate `1 / self` with relative error `< 2^-103.9`.
    #[inline]
    pub(super) fn inv(self) -> Self {
        let mut a = self.to_f64(true);
        let subnormal = a.abs() < f64::from_bits(0x0010_0000_0000_0000);
        if subnormal {
            a *= f64::from_bits(0x4340_0000_0000_0000);
        }
        let mut r = Self::from_f64(1.0 / a);
        if subnormal {
            r.ex += 53;
        }
        let mut q = self.mul(r);
        q.sgn = 1 - q.sgn;
        q = ONE.add(q);
        r.add(r.mul(q))
    }

    /// Approximate `self / denom` with relative error `< 2^-185.53`.
    #[inline]
    pub(super) fn div(self, denom: Self) -> Self {
        let y = denom.inv();
        let r = y.mul(self);
        let mut z = denom.mul(r);
        z.sgn = 1 - z.sgn;
        z = self.add(z);
        r.add(y.mul(z))
    }

    /// Approximate `num / denom` from two finite nonzero `f64` values.
    #[inline]
    pub(super) fn div_f64(num: f64, denom: f64) -> Self {
        Self::from_f64(num).div(Self::from_f64(denom))
    }
}
