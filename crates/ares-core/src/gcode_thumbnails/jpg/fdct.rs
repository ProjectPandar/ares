/*
 * jfdctint.c
 *
 * This file was part of the Independent JPEG Group's software:
 * Copyright (C) 1991-1996, Thomas G. Lane.
 * libjpeg-turbo Modifications:
 * Copyright (C) 2015, 2020, 2022, D. R. Commander.
 * For conditions of distribution and use, see the accompanying README.ijg
 * file.
 *
 * This file contains a slower but more accurate integer implementation of the
 * forward DCT (Discrete Cosine Transform).
 *
 * A 2-D DCT can be done by 1-D DCT on each row followed by 1-D DCT
 * on each column.  Direct algorithms are also available, but they are
 * much more complex and seem not to be any faster when reduced to code.
 *
 * This implementation is based on an algorithm described in
 *   C. Loeffler, A. Ligtenberg and G. Moschytz, "Practical Fast 1-D DCT
 *   Algorithms with 11 Multiplications", Proc. Int'l. Conf. on Acoustics,
 *   Speech, and Signal Processing 1989 (ICASSP '89), pp. 988-991.
 * The primary algorithm described there uses 11 multiplies and 29 adds.
 * We use their alternate method with 12 multiplies and 32 adds.
 * The advantage of this method is that no data path contains more than one
 * multiplication; this allows a very simple and accurate implementation in
 * scaled fixed-point arithmetic, with a minimal number of shifts.
 */
/*
 * jcdctmgr.c
 *
 * This file was part of the Independent JPEG Group's software:
 * Copyright (C) 1994-1996, Thomas G. Lane.
 * libjpeg-turbo Modifications:
 * Copyright (C) 1999-2006, MIYASAKA Masaru.
 * Copyright 2009 Pierre Ossman <ossman@cendio.se> for Cendio AB
 * Copyright (C) 2011, 2014-2015, 2022, 2024-2026, D. R. Commander.
 * For conditions of distribution and use, see the accompanying README.ijg
 * file.
 *
 * This file contains the forward-DCT management logic.
 * This code selects a particular DCT implementation to be used,
 * and it performs related housekeeping chores including coefficient
 * quantization.
 */
// Rust port copyright (C) 2026 Ares contributors.
// Fixed libjpeg-turbo3.2.0 thumbnail path; see README.ijg.

pub(super) struct Quantizer {
    divisors: [Divisor; 64],
}

struct Divisor {
    reciprocal: u32,
    correction: u32,
    shift: u32,
}

impl Quantizer {
    pub fn new(table: &[u8; 64]) -> Self {
        Self {
            divisors: table.map(|value| {
                let divisor = u32::from(value) * 8;
                let mut shift = 16 + (31 - divisor.leading_zeros());
                let numerator = 1_u32 << shift;
                let mut reciprocal = numerator / divisor;
                let remainder = numerator % divisor;
                let mut correction = divisor / 2;
                if remainder == 0 {
                    reciprocal >>= 1;
                    shift -= 1;
                } else if remainder <= divisor / 2 {
                    correction += 1;
                } else {
                    reciprocal += 1;
                }
                Divisor {
                    reciprocal,
                    correction,
                    shift,
                }
            }),
        }
    }

    pub fn block(&self, mut samples: [i32; 64]) -> [i32; 64] {
        for row in samples.as_chunks_mut::<8>().0 {
            *row = pass::<true>(*row);
        }
        for column in 0..8 {
            let values = pass::<false>(std::array::from_fn(|row| samples[row * 8 + column]));
            for (row, value) in values.into_iter().enumerate() {
                samples[row * 8 + column] = value;
            }
        }
        std::array::from_fn(|index| {
            let value = samples[index];
            let divisor = &self.divisors[index];
            let magnitude =
                ((value.unsigned_abs() + divisor.correction) * divisor.reciprocal) >> divisor.shift;
            value.signum() * magnitude as i32
        })
    }
}

fn descale(value: i32, shift: u32) -> i32 {
    (value + (1 << (shift - 1))) >> shift
}

/// `_jpeg_fdct_islow`: CONST_BITS=13, PASS1_BITS=2, 8-bit samples only.
fn pass<const ROW: bool>(values: [i32; 8]) -> [i32; 8] {
    let tmp0 = values[0] + values[7];
    let tmp7 = values[0] - values[7];
    let tmp1 = values[1] + values[6];
    let tmp6 = values[1] - values[6];
    let tmp2 = values[2] + values[5];
    let tmp5 = values[2] - values[5];
    let tmp3 = values[3] + values[4];
    let tmp4 = values[3] - values[4];
    let tmp10 = tmp0 + tmp3;
    let tmp13 = tmp0 - tmp3;
    let tmp11 = tmp1 + tmp2;
    let tmp12 = tmp1 - tmp2;
    let shift = if ROW { 11 } else { 15 };
    let z1_even = (tmp12 + tmp13) * 4433;
    let z5 = (tmp4 + tmp6 + tmp5 + tmp7) * 9633;
    let z1 = (tmp4 + tmp7) * -7373;
    let z2 = (tmp5 + tmp6) * -20995;
    let z3 = (tmp4 + tmp6) * -16069 + z5;
    let z4 = (tmp5 + tmp7) * -3196 + z5;
    [
        if ROW {
            (tmp10 + tmp11) << 2
        } else {
            descale(tmp10 + tmp11, 2)
        },
        descale(tmp7 * 12299 + z1 + z4, shift),
        descale(z1_even + tmp13 * 6270, shift),
        descale(tmp6 * 25172 + z2 + z3, shift),
        if ROW {
            (tmp10 - tmp11) << 2
        } else {
            descale(tmp10 - tmp11, 2)
        },
        descale(tmp5 * 16819 + z2 + z4, shift),
        descale(z1_even - tmp12 * 15137, shift),
        descale(tmp4 * 2446 + z1 + z3, shift),
    ]
}
