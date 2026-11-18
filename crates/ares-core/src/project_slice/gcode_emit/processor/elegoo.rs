//! `ElegooGCodeProcessorHelper.cpp`: M6211 decoding and Centauri purge timing.

use super::motion_util::word;

#[derive(Clone, Copy)]
pub(super) enum Model {
    Centauri,
    Centauri2,
    OtherElegoo,
}

pub(super) struct Purge {
    pub(super) filament: usize,
    pub(super) length: f32,
    pub(super) temperature: Option<f32>,
    pub(super) seconds: f32,
}

impl Model {
    pub(super) fn from_printer_model(model: &str) -> Option<Self> {
        if !model.get(..6)?.eq_ignore_ascii_case("elegoo") {
            return None;
        }
        Some(
            if model.eq_ignore_ascii_case("Elegoo Centauri Carbon")
                || model.eq_ignore_ascii_case("Elegoo Centauri")
            {
                Self::Centauri
            } else if model.eq_ignore_ascii_case("Elegoo Centauri Carbon 2")
                || model.eq_ignore_ascii_case("Elegoo Centauri 2")
            {
                Self::Centauri2
            } else {
                Self::OtherElegoo
            },
        )
    }

    pub(super) fn purge(self, code: &str, count: usize, [x, y]: [f64; 2]) -> Option<Purge> {
        if !matches!(
            code.split_ascii_whitespace().next(),
            Some("M6211" | "m6211")
        ) {
            return None;
        }
        let length = parameter(code, 'L').filter(|&value| value > 0.0)?;
        let tool = parameter(code, 'T').filter(|&value| value >= 0.0)?.round();
        if tool as usize >= count {
            return None;
        }
        let temperature = parameter(code, 'S');
        let seconds = match self {
            Self::Centauri => centauri_time(code, length, x, y),
            Self::Centauri2 => centauri2_time(code, length, temperature.unwrap_or(0.0)),
            Self::OtherElegoo => 0.0,
        };
        Some(Purge {
            filament: tool as usize,
            length,
            temperature,
            seconds,
        })
    }
}

fn parameter(code: &str, axis: char) -> Option<f32> {
    word(code, axis)
        .map(|value| value as f32)
        .filter(|value| value.is_finite())
}

fn clamped(code: &str, axis: char, default: f32, min: f32, max: f32) -> f32 {
    parameter(code, axis).unwrap_or(default).clamp(min, max)
}

fn extrusion_time(length: f32, feedrate: f32) -> f32 {
    length / feedrate * 60.0
}

/// `:51–86`: full-segment extrusion differs from the last partial segment.
fn centauri_time(code: &str, length: f32, x: f64, y: f64) -> f32 {
    let mut remaining = length.clamp(10.0, 1000.0);
    let cool = clamped(code, 'P', 5000.0, 0.0, 20000.0) * 0.001;
    let travel = (y.abs() + (x - 256.0).abs()) as f32 / 5000.0 * 60.0;
    let mut seconds = 18.2 + travel;
    while remaining > 0.0 {
        let segment = remaining.min(73.0);
        remaining -= segment;
        let extrusion = if segment >= 73.0 {
            extrusion_time(30.0, 500.0) + extrusion_time(35.0, 500.0) + extrusion_time(10.0, 400.0)
        } else {
            extrusion_time(segment, 500.0)
        };
        seconds += extrusion + extrusion_time(2.0, 400.0) + cool + 2.8;
    }
    seconds
}

/// `:40–49,89–124`: E15 on the old filament, then fixed-length S819 flushes.
fn centauri2_time(code: &str, length: f32, temperature: f32) -> f32 {
    let length = length.clamp(10.0, 1000.0);
    let segment = clamped(code, 'K', 75.0, 10.0, 300.0);
    let old_feed = clamped(code, 'M', 300.0, 10.0, 600.0);
    let new_feed = clamped(code, 'N', 300.0, 10.0, 600.0);
    let cool = clamped(code, 'P', 3000.0, 0.0, 20000.0) * 0.001;
    let flushes = ((length - 15.0).max(0.0) / segment).ceil().max(1.0);
    let flush_time = |feed, retract| {
        extrusion_time(segment - 10.0, feed)
            + extrusion_time(10.0, 400.0)
            + extrusion_time(retract, 1800.0)
            + cool
            + 5.0
    };
    let mut seconds = 31.0 + extrusion_time(length.min(15.0), old_feed);
    seconds += (flushes - 1.0) * flush_time(new_feed, 6.0);
    seconds += flush_time(new_feed * 0.8, 4.0);
    let r = clamped(code, 'R', temperature + 20.0, 185.0, 350.0);
    let s = clamped(code, 'S', 250.0, 185.0, 350.0);
    if s < r {
        seconds += (r - s) / 1.36;
    }
    seconds
}
