// Copyright (c) 2018-2026, Arm Limited.
// SPDX-License-Identifier: MIT
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

//! Arm optimized-routines `math/{sinf.c,cosf.c,sincosf.h,sincosf_data.c}` at
//! `20859d3fd9e797dde592c489c8ef7d383f778620`, for finite binary32 arguments.

use std::{f32::consts::FRAC_PI_4, f64::consts::FRAC_PI_2};

const SCALED_INV_HALF_PI: f64 = f64::from_bits(0x4164_5f30_6dc9_c883);
const SINE: [f64; 3] = [
    f64::from_bits(0xbfc5_5554_5995_a603),
    f64::from_bits(0x3f81_1076_0523_0bc4),
    f64::from_bits(0xbf29_94eb_3774_cf24),
];
const COSINE: [f64; 5] = [
    1.0,
    f64::from_bits(0xbfdf_ffff_fd0c_621c),
    f64::from_bits(0x3fa5_5553_e106_8f19),
    f64::from_bits(0xbf56_c087_e89a_359d),
    f64::from_bits(0x3ef9_9343_027b_f8c3),
];

pub(crate) fn sin(angle: f32) -> f32 {
    sample(angle, false)
}

pub(crate) fn cos(angle: f32) -> f32 {
    sample(angle, true)
}

pub(crate) fn sin_cos(angle: f32) -> (f32, f32) {
    (sample(angle, false), sample(angle, true))
}

fn sample(angle: f32, cosine: bool) -> f32 {
    const MAGNITUDE_TOP: u32 = 0x7ff0_0000;
    const TINY: u32 = 0x3980_0000; // 2^-12
    let bits = angle.to_bits();
    let top = bits & MAGNITUDE_TOP;
    if top < TINY {
        return if cosine { 1.0 } else { angle };
    }

    let x = f64::from(angle);
    let (x, quadrant, sign_quadrant) = if top < FRAC_PI_4.to_bits() & MAGNITUDE_TOP {
        (x, 0, 0)
    } else if top < 120.0_f32.to_bits() & MAGNITUDE_TOP {
        // Arm reduce_fast's scaled conversion and a single rounded subtraction.
        let quadrant = (((x * SCALED_INV_HALF_PI) as i32) + 0x80_0000) >> 24;
        (
            (-f64::from(quadrant)).mul_add(FRAC_PI_2, x),
            quadrant,
            quadrant,
        )
    } else {
        let (x, quadrant) = reduce_large(bits);
        // Large reduction ignores the input sign; restore it only for sine.
        let sign = if cosine { 0 } else { (bits >> 31) as i32 };
        (x, quadrant, quadrant + sign)
    };
    let x2 = x * x;
    let x = x * [1.0, -1.0, -1.0, 1.0][(sign_quadrant & 3) as usize];
    let use_cosine = (quadrant & 1 != 0) ^ cosine;
    if use_cosine {
        let sign = if sign_quadrant & 2 == 0 { 1.0 } else { -1.0 };
        let [c0, c1, c2, c3, c4] = COSINE.map(|coefficient| sign * coefficient);
        let x4 = x2 * x2;
        let high = x2.mul_add(c4, c3);
        let low = x2.mul_add(c1, c0);
        let x6 = x4 * x2;
        let base = x4.mul_add(c2, low);
        x6.mul_add(high, base) as f32
    } else {
        let [s1, s2, s3] = SINE;
        let x3 = x * x2;
        let high = x2.mul_add(s3, s2);
        let x5 = x3 * x2;
        let base = x3.mul_add(s1, x);
        x5.mul_add(high, base) as f32
    }
}

fn reduce_large(bits: u32) -> (f64, i32) {
    // Overlapping words of 4/pi, indexed by exponent; Arm's reduce_large.
    const INV_PIO4: [u32; 24] = [
        0xa2, 0xa2f9, 0xa2f983, 0xa2f9836e, 0xf9836e4e, 0x836e4e44, 0x6e4e4415, 0x4e441529,
        0x441529fc, 0x1529fc27, 0x29fc2757, 0xfc2757d1, 0x2757d1f5, 0x57d1f534, 0xd1f534dd,
        0xf534ddc0, 0x34ddc0db, 0xddc0db62, 0xc0db6295, 0xdb629599, 0x6295993c, 0x95993c43,
        0x993c4390, 0x3c439041,
    ];
    const PI_SCALED: f64 = f64::from_bits(0x3c19_21fb_5444_2d18); // pi * 2^-63

    let index = ((bits >> 26) & 15) as usize;
    let significand = ((bits & 0xff_ffff) | 0x80_0000) << ((bits >> 23) & 7);
    let high = u64::from(significand.wrapping_mul(INV_PIO4[index]));
    let middle = u64::from(significand) * u64::from(INV_PIO4[index + 4]);
    let low = u64::from(significand) * u64::from(INV_PIO4[index + 8]);
    let phase = ((high << 32) | (low >> 32)).wrapping_add(middle);
    let quadrant = phase.wrapping_add(1 << 61) >> 62;
    let residual = phase.wrapping_sub(quadrant << 62) as i64;
    (residual as f64 * PI_SCALED, quadrant as i32)
}
