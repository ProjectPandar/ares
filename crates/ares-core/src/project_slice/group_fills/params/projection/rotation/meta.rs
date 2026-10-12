//! Port of `calculate_infill_rotation_angle`'s template meta-language
//! (`OrcaSlicer src/libslic3r/Fill/Fill.cpp:60-203`), including its parsing
//! quirks (`strtod`/`strtol` prefix parsing, `*n` suffix repointing `cs` and
//! discarding earlier flag/unit characters, `!` one-time tokens, `_noop`
//! dumb cycles). Joint transforms map `search_string.find` (`Fill.cpp:62`)
//! indices onto the 16-case table (`Fill.cpp:189-202`).
//!
//! The `~`/`^` joints draw upstream's `rand()` (`Fill.cpp:199-200`): the
//! slicing path never calls `srand`, so the oracle produces glibc's
//! default-seeded TYPE_3 additive-feedback sequence, reproduced by
//! [`GlibcRandom`] per worker thread via [`thread_local`] (mirroring
//! upstream's process-global C `rand()` state).

use std::cell::RefCell;

use crate::{ObjectOptions, RegionOptions, project_slice::layers::PlannedLayer};

struct MetaLayerContext<'a> {
    pub(super) layers: &'a [PlannedLayer],
    /// `object->height()`: model bounding height = last layer `print_z`
    /// minus first layer `bottom_z` (object bottom on the plate).
    pub(super) object_height: f64,
    /// `object->config().layer_height`.
    pub(super) layer_height: f64,
    /// `default_region_config().{bottom,top}_shell_layers`
    /// (`Fill.cpp:125-127`); resolved from the current region — equal for
    /// the single-region objects this gate supports.
    pub(super) bottom_shell_layers: f64,
    pub(super) top_shell_layers: f64,
}

fn bottom_z(layer: &PlannedLayer) -> f64 {
    layer.print_z - layer.height
}

/// Shared context assembly + dispatch used by both the sparse/solid fill
/// projection and the ironing path: meta templates run the ported
/// interpreter, anything else runs the non-meta list path
/// (`Fill.cpp:206-210`). Returns the angle in degrees.
pub(in crate::project_slice) fn template_rotation_angle(
    template: &str,
    layers: &[PlannedLayer],
    object: &ObjectOptions,
    region: &RegionOptions,
    layer_id: usize,
) -> f64 {
    if is_meta_template(template) {
        let context = MetaLayerContext {
            layers,
            object_height: layers[layers.len() - 1].print_z - bottom_z(&layers[0]),
            layer_height: object.layer_height.0,
            bottom_shell_layers: f64::from(region.bottom_shell_layers.0),
            top_shell_layers: f64::from(region.top_shell_layers.0),
        };
        meta_rotation_angle(template, &context, layer_id)
    } else {
        super::simple_rotation_angle(template, layer_id)
    }
}

/// `search_string` index space of upstream's joint table (`Fill.cpp:62`).
const SEARCH_STRING: &[u8] = b"/NnZz$LlUuQq~^|#";

/// Mirrors `regex_search(template, [+\-%*@'"cm` + search_string + `])`
/// (`Fill.cpp:63-64`): any byte from this set selects the meta path.
fn is_meta_template(template: &str) -> bool {
    template.bytes().any(|byte| {
        matches!(
            byte,
            b'+' | b'-' | b'%' | b'*' | b'@' | b'\'' | b'"' | b'c' | b'm'
        ) || SEARCH_STRING.contains(&byte)
    })
}

/// C `strtod` prefix parse: `[-+]?digits[.digits][eE[+-]digits]`, plus the
/// `inf`/`nan` words are out of scope (templates do not produce them through
/// sanctioned config input). Returns `(value, bytes_consumed)`.
pub(super) fn strtod_prefix(bytes: &[u8]) -> (f64, usize) {
    let text = std::str::from_utf8(bytes).unwrap_or("");
    let bytes = text.as_bytes();
    let mut end = 0;
    if end < bytes.len() && matches!(bytes[end], b'+' | b'-') {
        end += 1;
    }
    while end < bytes.len() && (bytes[end].is_ascii_digit() || bytes[end] == b'.') {
        end += 1;
    }
    if end < bytes.len() && matches!(bytes[end], b'e' | b'E') {
        let mut probe = end + 1;
        if probe < bytes.len() && matches!(bytes[probe], b'+' | b'-') {
            probe += 1;
        }
        if probe < bytes.len() && bytes[probe].is_ascii_digit() {
            end = probe;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
        }
    }
    (text[..end].parse::<f64>().unwrap_or(0.0), end)
}

/// C `strtol(_, base 0)` restricted to the decimal templates used by `*n`.
fn strtol_prefix(text: &str) -> (i64, usize) {
    let bytes = text.as_bytes();
    let mut end = 0;
    if end < bytes.len() && matches!(bytes[end], b'+' | b'-') {
        end += 1;
    }
    let digits_start = end;
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    if end == digits_start {
        return (0, end);
    }
    (text[..end].parse::<i64>().unwrap_or(0), end)
}

/// glibc `rand()` under default seed 1: the TYPE_3 additive feedback
/// generator over a 31-word state (`deg = 31`, `sep = 3`), `initstate`
/// discarding the first `10 * deg` outputs. Verified against the canonical
/// default sequence `1804289383, 846930886, 1681692777, …`.
struct GlibcRandom {
    r: [i32; 31],
    f: usize,
    ridx: usize,
}

impl GlibcRandom {
    fn new() -> Self {
        let mut r = [0i32; 31];
        r[0] = 1;
        for i in 1..31 {
            let hi = r[i - 1] / 127773;
            let lo = r[i - 1] % 127773;
            let mut word = 16807 * lo - 2836 * hi;
            if word < 0 {
                word += 2147483647;
            }
            r[i] = word;
        }
        let mut rng = Self { r, f: 3, ridx: 0 };
        for _ in 0..310 {
            rng.next_i31();
        }
        rng
    }

    fn next_i31(&mut self) -> i32 {
        // *fptr += *rptr; both pointers advance in lockstep through the
        // 31-word ring, rptr trailing fptr by `sep` = 3.
        self.r[self.f] = self.r[self.f].wrapping_add(self.r[self.ridx]);
        let value = ((self.r[self.f] as u32) >> 1) as i32;
        self.f = (self.f + 1) % 31;
        self.ridx = (self.ridx + 1) % 31;
        value
    }

    fn next_unit(&mut self) -> f64 {
        f64::from(self.next_i31()) / f64::from(i32::MAX)
    }
}

thread_local! {
    static RAND_STATE: RefCell<GlibcRandom> = RefCell::new(GlibcRandom::new());
}

/// `Fill.cpp:60-203` meta path. Returns the angle in **degrees** — upstream
/// only applies `deg2rad` to `angle_start + angle_add * negvalue` at the end,
/// so this function yields exactly that pre-conversion value.
fn meta_rotation_angle(template: &str, context: &MetaLayerContext<'_>, layer_id: usize) -> f64 {
    let layers = context.layers;
    let tokens: Vec<&str> = template
        .split(|character: char| character == ',' || character.is_whitespace())
        .filter(|token| !token.is_empty())
        .collect();
    let mut state = UnitState {
        t: 0,
        repeats: 0,
        angle_add: 0.0,
        angle_start: 0.0,
        limit_fill_z: bottom_z(&layers[0]),
        start_fill_z: 0.0,
        noop: false,
        fill_form: usize::MAX,
        negative: false,
        stop: vec![false; tokens.len()],
    };
    state.start_fill_z = state.limit_fill_z;
    let mut angle = 0.0;

    for (i, layer) in layers.iter().enumerate().take(layer_id + 1) {
        let fill_z = bottom_z(layer);
        if state.limit_fill_z < layer.slice_z {
            if state.repeats > 0 {
                // `Fill.cpp:88-91`: extend the window, consume one repeat.
                state.limit_fill_z += state.limit_fill_z - state.start_fill_z;
                state.start_fill_z = fill_z;
                state.repeats -= 1;
            } else {
                state.start_fill_z = fill_z;
                state.limit_fill_z = layer.print_z;
                state.fill_form = usize::MAX;
                read_unit_tokens(&tokens, i, fill_z, context, &mut state);
            }
        }
        let top_z = layer.print_z;
        let negative = state.negative;
        let limit_fill_z = state.limit_fill_z;
        let mut negvalue = if negative {
            limit_fill_z - top_z
        } else {
            top_z - state.start_fill_z
        } / (limit_fill_z - state.start_fill_z);
        let pi = std::f64::consts::PI;
        negvalue = match state.fill_form {
            1 => negvalue - (negvalue * pi * 2.0).sin() / (pi * 2.0),
            2 => negvalue - (negvalue * pi * 2.0).sin() / (pi * 4.0),
            3 => negvalue + (negvalue * pi * 2.0).sin() / (pi * 2.0),
            4 => negvalue + (negvalue * pi * 2.0).sin() / (pi * 4.0),
            5 => (negvalue * 2.0 - 1.0).asin() / pi + 0.5,
            6 => (negvalue * pi / 2.0).sin(),
            7 => 1.0 - (negvalue * pi / 2.0).cos(),
            8 => 1.0 - (1.0 - negvalue).powf(2.0),
            9 => (1.0 - negvalue).powf(2.0),
            10 => 1.0 - (1.0 - negvalue).powf(3.0),
            11 => (1.0 - negvalue).powf(3.0),
            12 => RAND_STATE.with(|state| state.borrow_mut().next_unit()),
            13 => negvalue + RAND_STATE.with(|state| state.borrow_mut().next_unit()) - 0.5,
            14 => 0.5,
            15 if negative => 0.0,
            15 => 1.0,
            _ => negvalue,
        };
        angle = state.angle_start + state.angle_add * negvalue;
    }
    angle
}

/// `Fill.cpp:84-99` interpreter registers, persisted across layers.
struct UnitState {
    /// Token cursor (`t`).
    t: usize,
    /// Pending window repetitions (`repeats`).
    repeats: i64,
    /// Incremental increment + accumulated absolute base
    /// (`angle_add`, `angle_start`).
    angle_add: f64,
    angle_start: f64,
    /// Window bounds (`limit_fill_z`, `start_fill_z`).
    limit_fill_z: f64,
    start_fill_z: f64,
    /// `_noop` dumb cycle, `fill_form` `search_string` index,
    /// `_negative` reverse window, `stop` one-time flags.
    noop: bool,
    fill_form: usize,
    negative: bool,
    stop: Vec<bool>,
}

/// `Fill.cpp:100-183`: read the next instruction unit (and any following
/// dumb/stopped units) for the window starting at `fill_z` on layer index
/// `i`, updating all interpreter registers.
fn read_unit_tokens(
    tokens: &[&str],
    i: usize,
    fill_z: f64,
    context: &MetaLayerContext<'_>,
    state: &mut UnitState,
) {
    loop {
        if !state.stop[state.t] {
            read_one_unit(tokens, i, fill_z, context, state);
        }
        state.t += 1;
        if state.t >= tokens.len() {
            state.t = 0;
        }
        // `Fill.cpp:178-182`: continue while the next token is a stopped or
        // dumb instruction, unless every token is consumed.
        if state.stop.iter().all(|stopped| *stopped)
            || !((state.t != 0 && state.noop) || state.stop[state.t])
        {
            break;
        }
    }
}

/// `Fill.cpp:131-143`: step count × length unit → z extent, where `mm`
/// leaves the value untouched, `m` multiplies by 1000 and the other units
/// map to inches/feet/cm/layers/percent-of-height. Note upstream's `%`
/// uses `height * 1e-8` (a factor of 100 smaller than a literal percent
/// reading; preserved verbatim).
fn unit_length(bytes: &[u8], cs: usize, angle_steps: f64, context: &MetaLayerContext<'_>) -> f64 {
    match bytes[cs] {
        b'%' => angle_steps * context.object_height * 1e-8,
        b'#' => angle_steps * context.layer_height,
        b'\'' => angle_steps * 12.0 * 25.4,
        b'"' => angle_steps * 25.4,
        b'c' => angle_steps * 10.0,
        b'm' if bytes.get(cs + 1) == Some(&b'm') => angle_steps,
        b'm' => angle_steps * 1000.0,
        _ => 0.0,
    }
}

/// `Fill.cpp:101-172`: a single instruction unit — angle value + optional
/// `%`, `*n` repeats, length units, `B`/`T` shell counts and interpolation
/// joint — updating the accumulated base, pending repeats and window end.
fn read_one_unit(
    tokens: &[&str],
    i: usize,
    fill_z: f64,
    context: &MetaLayerContext<'_>,
    state: &mut UnitState,
) {
    let token = tokens[state.t];
    let bytes = token.as_bytes();
    state.noop = false;
    let mut absolute = false;
    state.negative = false;
    state.angle_start += state.angle_add;
    // Upstream `angle_steps = 1` is always overwritten by the B/T/strtod
    // arms before any read; declare without the dead initializer.
    let mut angle_steps: f64;
    state.repeats = 1;
    if token.contains('!') {
        state.stop[state.t] = true;
    }
    let mut cs = 0usize;
    if bytes[0].is_ascii_digit() {
        absolute = true;
    }
    let (parsed, consumed) = strtod_prefix(&bytes[cs..]);
    state.angle_add = parsed;
    cs += consumed;
    if bytes.get(cs) == Some(&b'%') {
        state.angle_add *= 3.6;
        cs += 1;
    }
    // `Fill.cpp:121-122`: `strtol` from after `*` also repoints `cs`,
    // discarding earlier unit chars.
    if let Some(star) = token.find('*') {
        let (value, consumed) = strtol_prefix(&token[star + 1..]);
        state.repeats = value;
        cs = star + 1 + consumed;
    }
    if state.repeats > 0 {
        match bytes.get(cs) {
            Some(b'B') => angle_steps = context.bottom_shell_layers,
            Some(b'T') => angle_steps = context.top_shell_layers,
            _ => {
                if let Some(&byte) = bytes.get(cs)
                    && let Some(index) = SEARCH_STRING.iter().position(|item| *item == byte)
                {
                    state.fill_form = index;
                    cs += 1;
                }
                state.negative = bytes.get(cs) == Some(&b'-');
                let (steps, consumed) = strtod_prefix(&bytes[cs..]);
                angle_steps = steps.abs();
                cs += consumed;
                if angle_steps != 0.0 && bytes.get(cs).is_some_and(|byte| *byte != b'!') {
                    state.limit_fill_z = unit_length(bytes, cs, angle_steps, context) + fill_z;
                    angle_steps = 0.0;
                }
            }
        }
        if angle_steps != 0.0 {
            if state.fill_form == usize::MAX && !absolute {
                state.angle_add *= angle_steps as i64 as f64;
            }
            let idx = i + (angle_steps - 1.0).max(0.0) as usize;
            let sdx = idx.saturating_sub(context.layers.len());
            let idx = idx.min(context.layers.len() - 1);
            state.limit_fill_z = context.layers[idx].print_z + sdx as f64 * context.layer_height;
        }
        state.repeats = (state.repeats - 1).max(0);
    } else {
        state.noop = true;
    }
    if absolute {
        state.angle_start = state.angle_add;
        state.angle_add = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glibc_default_sequence_matches_oracle() {
        let mut rng = GlibcRandom::new();
        let draws: Vec<i32> = (0..3).map(|_| rng.next_i31()).collect();
        // Canonical glibc rand(1) draws.
        assert_eq!(draws, [1804289383, 846930886, 1681692777]);
    }
}
