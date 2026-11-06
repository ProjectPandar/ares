//! `libslic3r/FlushVolCalc.cpp` and pure RGBA/HSV CLI dependencies (8500fcd).
pub(super) mod color_distance;
mod predictor;

use std::f64::consts::PI;

pub(super) fn rgba(text: &str) -> [u8; 4] {
    // BitmapCache::parse_color4: initialized opaque black, partial parse on error.
    let mut result = [0, 0, 0, 255];
    let bytes = text.as_bytes();
    if !matches!(bytes.len(), 7 | 9) || bytes[0] != b'#' {
        return result;
    }
    for (i, pair) in bytes[1..].as_chunks::<2>().0.iter().enumerate() {
        let digit = |byte: u8| char::from(byte).to_digit(16);
        let (Some(high), Some(low)) = (digit(pair[0]), digit(pair[1])) else {
            break;
        };
        result[i] = (high * 16 + low) as u8;
    }
    result
}

pub(super) fn calculate(minimum: i32, dataset: i32, from: [u8; 4], to: [u8; 4]) -> i32 {
    let rgb = |[r, g, b, a]: [u8; 4]| if a == 0 { [255; 3] } else { [r, g, b] };
    let (from, to) = (rgb(from), rgb(to));
    let prediction = predictor::predict(dataset, from, to);
    if dataset != 0
        && let Some(volume) = prediction
    {
        return (volume as i32).min(20000);
    }
    let mut volume = if dataset == 0
        && let Some(volume) = prediction
    {
        // calc_flush_vol_rgb returns int before the outer minimum addition.
        volume as i32
    } else {
        geometric_volume(from, to)
    } as f32;
    let from_lumi = luminance(from.map(|x| f32::from(x) / 255.0));
    let to_lumi = luminance(to.map(|x| f32::from(x) / 255.0));
    if dataset != 0 && from_lumi > 180.0 / 255.0 && to_lumi < 75.0 / 255.0 {
        volume = (f64::from(volume) * 1.3) as f32;
    }
    volume += minimum as f32;
    (volume as i32).min(20000)
}

fn luminance([r, g, b]: [f32; 3]) -> f32 {
    (f64::from(r) * 0.3 + f64::from(g) * 0.59 + f64::from(b) * 0.11) as f32
}

fn hsv([r, g, b]: [f32; 3]) -> [f32; 3] {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let hue = if f64::from(delta.abs()) < 0.001 {
        0.0
    } else if max == r {
        (60.0 * (f64::from((g - b) / delta) % 6.0)) as f32
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    let saturation = if f64::from(max.abs()) < 0.001 {
        0.0
    } else {
        delta / max
    };
    [hue, saturation, max]
}

fn radians(degree: f32) -> f32 {
    (f64::from(degree / 180.0) * PI) as f32
}

fn geometric_volume(from: [u8; 3], to: [u8; 3]) -> i32 {
    let from = from.map(|x| f32::from(x) / 255.0);
    let to = to.map(|x| f32::from(x) / 255.0);
    let [h1, s1, v1] = hsv(from);
    let [h2, s2, v2] = hsv(to);
    let (h1, h2) = (radians(h1), radians(h2));
    // The upstream second terms use global cos/sin (double), then narrow.
    let dx = (f64::from(h1.cos() * s1 * v1) - f64::from(h2).cos() * f64::from(s2) * f64::from(v2))
        as f32;
    let dy = (f64::from(h1.sin() * s1 * v1) - f64::from(h2).sin() * f64::from(s2) * f64::from(v2))
        as f32;
    let mut hs_distance = (dx * dx + dy * dy).sqrt().min(1.2);
    let (from_lumi, to_lumi) = (luminance(from), luminance(to));
    let lumi_flush = if to_lumi >= from_lumi {
        (to_lumi - from_lumi).powf(0.7) * 560.0
    } else {
        let value = (0.67 * f64::from(v2) + 0.33 * f64::from(v1)) as f32;
        hs_distance = hs_distance.min(value);
        (from_lumi - to_lumi) * 80.0
    };
    let hs_flush = 230.0 * hs_distance;
    (hs_flush * hs_flush + lumi_flush * lumi_flush
        - 2.0 * hs_flush * lumi_flush * radians(120.0).cos())
    .sqrt()
    .max(60.0) as i32
}
