//! `libslic3r/FlushVolPredictor.cpp::RGB2LAB/calc_color_distance` (8500fcd).
use std::f64::consts::PI;

fn lab([r, g, b]: [u8; 3]) -> [f64; 3] {
    let gamma = |byte: u8| {
        let x = f64::from(byte) / 255.0;
        if x > 0.04045 {
            ((x + 0.055) / 1.055).powf(2.4) * 100.0
        } else {
            x / 12.92 * 100.0
        }
    };
    let (r, g, b) = (gamma(r), gamma(g), gamma(b));
    let pivot = |t: f64| {
        if t > f64::from(0.008856_f32) {
            t.powf(1.0 / 3.0)
        } else {
            7.787 * t + 0.137931
        }
    };
    let x = pivot((0.412453 * r + 0.357580 * g + 0.180423 * b) / 95.0489);
    let y = pivot((0.212671 * r + 0.715160 * g + 0.072169 * b) / 100.0);
    let z = pivot((0.019334 * r + 0.119193 * g + 0.950227 * b) / 108.8840);
    [116.0 * y - 16.0, 500.0 * (x - y), 200.0 * (y - z)]
}

pub(crate) fn distance(from: [u8; 3], to: [u8; 3]) -> f32 {
    let [l1, a1, b1] = lab(from);
    let [l2, a2, b2] = lab(to);
    let c1 = (a1 * a1 + b1 * b1).sqrt();
    let c2 = (a2 * a2 + b2 * b2).sqrt();
    let c_mean = (c1 + c2) / 2.0;
    let pow_c_mean = c_mean.powf(7.0);
    let pow_25 = 25.0_f64.powf(7.0);
    let g = 0.5 * (1.0 - (pow_c_mean / (pow_c_mean + pow_25)).sqrt());
    let pa1 = (1.0 + g) * a1;
    let pa2 = (1.0 + g) * a2;
    let pc1 = (pa1 * pa1 + b1 * b1).sqrt();
    let pc2 = (pa2 * pa2 + b2 * b2).sqrt();
    let hue = |a: f64, b: f64| {
        if a == 0.0 && b == 0.0 {
            0.0
        } else {
            let angle = b.atan2(a);
            if angle < 0.0 { angle + PI * 2.0 } else { angle }
        }
    };
    let (ph1, ph2) = (hue(pa1, b1), hue(pa2, b2));
    let dl = l2 - l1;
    let dc = pc2 - pc1;
    let dh = if pc1 * pc2 == 0.0 {
        0.0
    } else {
        let mut delta = ph2 - ph1;
        if delta < -PI {
            delta += PI * 2.0;
        } else if delta > PI {
            delta -= PI * 2.0;
        }
        2.0 * (pc1 * pc2).sqrt() * (delta / 2.0).sin()
    };
    let pl_mean = (l1 + l2) / 2.0;
    let pc_mean = (pc1 + pc2) / 2.0;
    let ph_sum = ph1 + ph2;
    let ph_mean = if pc1 * pc2 == 0.0 {
        ph_sum
    } else if (ph1 - ph2).abs() <= PI {
        ph_sum / 2.0
    } else if ph_sum < PI * 2.0 {
        (ph_sum + PI * 2.0) / 2.0
    } else {
        (ph_sum - PI * 2.0) / 2.0
    };
    let rad = |degrees: f64| degrees * PI / 180.0;
    let t = 1.0 - 0.17 * (ph_mean - rad(30.0)).cos()
        + 0.24 * (2.0 * ph_mean).cos()
        + 0.32 * (3.0 * ph_mean + rad(6.0)).cos()
        - 0.2 * (4.0 * ph_mean - rad(63.0)).cos();
    let dtheta = rad(30.0) * (-((ph_mean - rad(275.0)) / rad(25.0)).powf(2.0)).exp();
    let pow_pc = pc_mean.powf(7.0);
    let rc = 2.0 * (pow_pc / (pow_pc + pow_25)).sqrt();
    let pow_pl = (pl_mean - 50.0).powf(2.0);
    let sl = 1.0 + 0.015 * pow_pl / (20.0 + pow_pl).sqrt();
    let sc = 1.0 + 0.045 * pc_mean;
    let sh = 1.0 + 0.015 * pc_mean * t;
    let rt = -(2.0 * dtheta).sin() * rc;
    ((dl / sl).powf(2.0) + (dc / sc).powf(2.0) + (dh / sh).powf(2.0) + rt * (dc / sc) * (dh / sh))
        .sqrt() as f32
}
