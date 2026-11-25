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
//! `20859d3fd9e797dde592c489c8ef7d383f778620`, restricted to wrapped fillet angles.

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

pub(super) fn sin_cos(angle: f32) -> (f32, f32) {
    (sample(angle, false), sample(angle, true))
}

fn sample(angle: f32, cosine: bool) -> f32 {
    const MAGNITUDE_TOP: u32 = 0x7ff0_0000;
    const TINY: u32 = 0x3980_0000; // 2^-12
    let top = angle.to_bits() & MAGNITUDE_TOP;
    if top < TINY {
        return if cosine { 1.0 } else { angle };
    }

    let x = f64::from(angle);
    let (x, quadrant) = if top < FRAC_PI_4.to_bits() & MAGNITUDE_TOP {
        (x, 0)
    } else {
        // Arm reduce_fast's scaled conversion and a single rounded subtraction.
        let quadrant = (((x * SCALED_INV_HALF_PI) as i32) + 0x80_0000) >> 24;
        ((-f64::from(quadrant)).mul_add(FRAC_PI_2, x), quadrant)
    };
    let x2 = x * x;
    let x = x * [1.0, -1.0, -1.0, 1.0][(quadrant & 3) as usize];
    let use_cosine = (quadrant & 1 != 0) ^ cosine;
    if use_cosine {
        let sign = if quadrant & 2 == 0 { 1.0 } else { -1.0 };
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
