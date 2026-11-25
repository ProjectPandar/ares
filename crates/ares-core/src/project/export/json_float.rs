// Port of OrcaSlicer's deps_src/nlohmann/detail/conversions/to_chars.hpp.
// Grisu2: Copyright (c) 2009 Florian Loitsch.
// JSON for Modern C++: Copyright (c) 2013-2019 Niels Lohmann.
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
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::io::{self, Write};

use serde_json::ser::Formatter;

pub(super) struct PlateJsonFormatter;

impl Formatter for PlateJsonFormatter {
    fn write_f32<W: Write + ?Sized>(&mut self, writer: &mut W, value: f32) -> io::Result<()> {
        self.write_f64(writer, f64::from(value))
    }

    fn write_f64<W: Write + ?Sized>(&mut self, writer: &mut W, value: f64) -> io::Result<()> {
        // As in dump_float (serializer.hpp:768–795), non-finite values are
        // emitted as null; serde_json handles that before calling this hook.
        let mut buffer = [0; 24]; // sign, up to 17 digits, point and e±ddd
        let start = usize::from(value.is_sign_negative());
        if start != 0 {
            buffer[0] = b'-';
        }
        if value == 0.0 {
            buffer[start..start + 3].copy_from_slice(b"0.0");
            return writer.write_all(&buffer[..start + 3]);
        }
        let digits = &mut buffer[start..];
        let (len, exponent) = grisu2(digits, value.abs());
        let end = start + format_buffer(digits, len, exponent);
        writer.write_all(&buffer[..end])
    }
}

#[derive(Clone, Copy)]
struct DiyFp {
    significand: u64,
    exponent: i32,
}

impl DiyFp {
    fn new(significand: u64, exponent: i32) -> Self {
        Self {
            significand,
            exponent,
        }
    }

    fn normalized(self) -> Self {
        let shift = self.significand.leading_zeros();
        Self::new(self.significand << shift, self.exponent - shift as i32)
    }

    fn normalized_to(self, exponent: i32) -> Self {
        Self::new(self.significand << (self.exponent - exponent), exponent)
    }

    fn mul(self, rhs: Self) -> Self {
        let product = u128::from(self.significand) * u128::from(rhs.significand);
        Self::new(
            ((product + (1 << 63)) >> 64) as u64,
            self.exponent + rhs.exponent + 64,
        )
    }
}

// to_chars.hpp:181–247: the two boundaries are midpoints to the adjacent
// binary64 values. The lower midpoint is closer at normal powers of two.
fn boundaries(value: f64) -> (DiyFp, DiyFp, DiyFp) {
    const HIDDEN: u64 = 1 << 52;
    let bits = value.to_bits();
    let exponent = (bits >> 52) as i32;
    let fraction = bits & (HIDDEN - 1);
    let v = if exponent == 0 {
        DiyFp::new(fraction, -1074)
    } else {
        DiyFp::new(fraction | HIDDEN, exponent - 1075)
    };
    let upper = DiyFp::new(2 * v.significand + 1, v.exponent - 1).normalized();
    let lower = if fraction == 0 && exponent > 1 {
        DiyFp::new(4 * v.significand - 1, v.exponent - 2)
    } else {
        DiyFp::new(2 * v.significand - 1, v.exponent - 1)
    };
    (lower.normalized_to(upper.exponent), v.normalized(), upper)
}

// to_chars.hpp:320–479: normalized cached powers (significand, exponent).
const CACHED_POWERS: [(u64, i32); 79] = [
    (0xab70_fe17_c79a_c6ca, -1060),
    (0xff77_b1fc_bebc_dc4f, -1034),
    (0xbe56_91ef_416b_d60c, -1007),
    (0x8dd0_1fad_907f_fc3c, -980),
    (0xd351_5c28_3155_9a83, -954),
    (0x9d71_ac8f_ada6_c9b5, -927),
    (0xea9c_2277_23ee_8bcb, -901),
    (0xaecc_4991_4078_536d, -874),
    (0x823c_1279_5db6_ce57, -847),
    (0xc210_9436_4dfb_5637, -821),
    (0x9096_ea6f_3848_984f, -794),
    (0xd774_85cb_2582_3ac7, -768),
    (0xa086_cfcd_97bf_97f4, -741),
    (0xef34_0a98_172a_ace5, -715),
    (0xb238_67fb_2a35_b28e, -688),
    (0x84c8_d4df_d2c6_3f3b, -661),
    (0xc5dd_4427_1ad3_cdba, -635),
    (0x936b_9fce_bb25_c996, -608),
    (0xdbac_6c24_7d62_a584, -582),
    (0xa3ab_6658_0d5f_daf6, -555),
    (0xf3e2_f893_dec3_f126, -529),
    (0xb5b5_ada8_aaff_80b8, -502),
    (0x8762_5f05_6c7c_4a8b, -475),
    (0xc9bc_ff60_34c1_3053, -449),
    (0x964e_858c_91ba_2655, -422),
    (0xdff9_7724_7029_7ebd, -396),
    (0xa6df_bd9f_b8e5_b88f, -369),
    (0xf8a9_5fcf_8874_7d94, -343),
    (0xb944_7093_8fa8_9bcf, -316),
    (0x8a08_f0f8_bf0f_156b, -289),
    (0xcdb0_2555_6531_31b6, -263),
    (0x993f_e2c6_d07b_7fac, -236),
    (0xe45c_10c4_2a2b_3b06, -210),
    (0xaa24_2499_6973_92d3, -183),
    (0xfd87_b5f2_8300_ca0e, -157),
    (0xbce5_0864_9211_1aeb, -130),
    (0x8cbc_cc09_6f50_88cc, -103),
    (0xd1b7_1758_e219_652c, -77),
    (0x9c40_0000_0000_0000, -50),
    (0xe8d4_a510_0000_0000, -24),
    (0xad78_ebc5_ac62_0000, 3),
    (0x813f_3978_f894_0984, 30),
    (0xc097_ce7b_c907_15b3, 56),
    (0x8f7e_32ce_7bea_5c70, 83),
    (0xd5d2_38a4_abe9_8068, 109),
    (0x9f4f_2726_179a_2245, 136),
    (0xed63_a231_d4c4_fb27, 162),
    (0xb0de_6538_8cc8_ada8, 189),
    (0x83c7_088e_1aab_65db, 216),
    (0xc45d_1df9_4271_1d9a, 242),
    (0x924d_692c_a61b_e758, 269),
    (0xda01_ee64_1a70_8dea, 295),
    (0xa26d_a399_9aef_774a, 322),
    (0xf209_787b_b47d_6b85, 348),
    (0xb454_e4a1_79dd_1877, 375),
    (0x865b_8692_5b9b_c5c2, 402),
    (0xc835_53c5_c896_5d3d, 428),
    (0x952a_b45c_fa97_a0b3, 455),
    (0xde46_9fbd_99a0_5fe3, 481),
    (0xa59b_c234_db39_8c25, 508),
    (0xf6c6_9a72_a398_9f5c, 534),
    (0xb7dc_bf53_54e9_bece, 561),
    (0x88fc_f317_f222_41e2, 588),
    (0xcc20_ce9b_d35c_78a5, 614),
    (0x9816_5af3_7b21_53df, 641),
    (0xe2a0_b5dc_971f_303a, 667),
    (0xa8d9_d153_5ce3_b396, 694),
    (0xfb9b_7cd9_a4a7_443c, 720),
    (0xbb76_4c4c_a7a4_4410, 747),
    (0x8bab_8eef_b640_9c1a, 774),
    (0xd01f_ef10_a657_842c, 800),
    (0x9b10_a4e5_e991_3129, 827),
    (0xe710_9bfb_a19c_0c9d, 853),
    (0xac28_20d9_623b_f429, 880),
    (0x8044_4b5e_7aa7_cf85, 907),
    (0xbf21_e440_03ac_dd2d, 933),
    (0x8e67_9c2f_5e44_ff8f, 960),
    (0xd433_179d_9c8c_b841, 986),
    (0x9e19_db92_b4e3_1ba9, 1013),
];

fn cached_power(exponent: i32) -> (DiyFp, i32) {
    // ceil((-60 - exponent - 1) * log10(2)) over binary64's exponent range.
    let f = -61 - exponent;
    let k = f * 78913 / (1 << 18) + i32::from(f > 0);
    let index = (300 + k + 7) / 8;
    let (significand, exponent) = CACHED_POWERS[index as usize];
    (DiyFp::new(significand, exponent), -300 + index * 8)
}

// to_chars.hpp:820–911: shrink the scaled interval by one unit on each side
// to cover multiplication and cached power rounding errors.
fn grisu2(buffer: &mut [u8], value: f64) -> (usize, i32) {
    let (lower, value, upper) = boundaries(value);
    let (power, exponent) = cached_power(upper.exponent);
    let scaled_value = value.mul(power);
    let scaled_lower = lower.mul(power);
    let scaled_upper = upper.mul(power);
    generate_digits(
        buffer,
        scaled_lower.significand + 1,
        scaled_value.significand,
        DiyFp::new(scaled_upper.significand - 1, scaled_upper.exponent),
        -exponent,
    )
}

// to_chars.hpp:538–564: stop at the closest candidate inside the interval.
fn round_last_digit(last: &mut u8, distance: u64, span: u64, mut rest: u64, step: u64) {
    while rest < distance
        && span - rest >= step
        && (rest + step < distance || distance - rest > rest + step - distance)
    {
        *last -= 1;
        rest += step;
    }
}

// to_chars.hpp:579–812: emit upper-bound digits left to right until the
// discarded remainder fits inside the interval.
fn generate_digits(
    buffer: &mut [u8],
    lower: u64,
    value: u64,
    upper: DiyFp,
    mut exponent: i32,
) -> (usize, i32) {
    let mut span = upper.significand - lower;
    let mut distance = upper.significand - value;
    let shift = -upper.exponent;
    let one = 1_u64 << shift;
    let mask = one - 1;
    let mut integral = upper.significand >> shift;
    let mut fraction = upper.significand & mask;
    let mut len = 0;
    let mut remaining = integral.ilog10();
    let mut place = 10_u64.pow(remaining);
    while place != 0 {
        buffer[len] = b'0' + (integral / place) as u8;
        len += 1;
        integral %= place;
        let rest = (integral << shift) + fraction;
        if rest <= span {
            round_last_digit(&mut buffer[len - 1], distance, span, rest, place << shift);
            return (len, exponent + remaining as i32);
        }
        place /= 10;
        remaining = remaining.saturating_sub(1);
    }
    loop {
        fraction *= 10;
        buffer[len] = b'0' + (fraction >> shift) as u8;
        len += 1;
        fraction &= mask;
        exponent -= 1;
        span *= 10;
        distance *= 10;
        if fraction <= span {
            round_last_digit(&mut buffer[len - 1], distance, span, fraction, one);
            return (len, exponent);
        }
    }
}

// to_chars.hpp:920–1106: use the same fixed/scientific cutovers as nlohmann.
fn format_buffer(buffer: &mut [u8], len: usize, exponent: i32) -> usize {
    let point = len as i32 + exponent;
    if point >= len as i32 && point <= 15 {
        let point = point as usize;
        buffer[len..point].fill(b'0');
        buffer[point..point + 2].copy_from_slice(b".0");
        point + 2
    } else if (1..=15).contains(&point) {
        let point = point as usize;
        buffer.copy_within(point..len, point + 1);
        buffer[point] = b'.';
        len + 1
    } else if (-3..=0).contains(&point) {
        let start = 2 + (-point) as usize;
        buffer.copy_within(..len, start);
        buffer[..2].copy_from_slice(b"0.");
        buffer[2..start].fill(b'0');
        start + len
    } else {
        let mut end = len;
        if len > 1 {
            buffer.copy_within(1..len, 2);
            buffer[1] = b'.';
            end += 1;
        }
        let exponent = point - 1;
        buffer[end] = b'e';
        buffer[end + 1] = if exponent < 0 { b'-' } else { b'+' };
        end += 2;
        let mut magnitude = exponent.unsigned_abs() as u16;
        if magnitude >= 100 {
            buffer[end] = b'0' + (magnitude / 100) as u8;
            end += 1;
            magnitude %= 100;
        }
        buffer[end] = b'0' + (magnitude / 10) as u8;
        buffer[end + 1] = b'0' + (magnitude % 10) as u8;
        end + 2
    }
}
