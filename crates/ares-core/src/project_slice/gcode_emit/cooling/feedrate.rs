mod parse;
mod rewrite;
mod slowdown;

const TYPE_EXTRUDE_END: u32 = 1 << 1;
const TYPE_G0: u32 = 1 << 4;
const TYPE_G1: u32 = 1 << 5;
const TYPE_ADJUSTABLE: u32 = 1 << 6;
const TYPE_EXTERNAL_PERIMETER: u32 = 1 << 7;
const TYPE_HAS_F: u32 = 1 << 8;
const TYPE_WIPE: u32 = 1 << 9;
const TYPE_G4: u32 = 1 << 10;
const TYPE_G92: u32 = 1 << 11;
const TYPE_G2: u32 = 1 << 12;
const TYPE_G3: u32 = 1 << 13;

#[derive(Clone, Copy)]
pub(super) struct Config {
    pub(super) enabled: bool,
    pub(super) target_time: f32,
    pub(super) minimum_speed: f32,
    pub(super) keep_outer_wall_speed: bool,
    pub(super) relative_e: bool,
}

pub(super) struct State {
    config: Config,
    position: [f32; 7],
}

impl State {
    pub(super) fn new(config: Config, travel_speed: f64) -> Self {
        let mut position = [0.0; 7];
        position[4] = travel_speed as f32;
        Self { config, position }
    }
}

#[derive(Clone, Copy)]
struct CoolingLine {
    kind: u32,
    start: usize,
    end: usize,
    length: f32,
    feedrate: f32,
    time: f32,
    maximum_time: f32,
    slowed: bool,
}

impl CoolingLine {
    const fn new(kind: u32, start: usize, end: usize) -> Self {
        Self {
            kind,
            start,
            end,
            length: 0.0,
            feedrate: 0.0,
            time: 0.0,
            maximum_time: 0.0,
            slowed: false,
        }
    }

    fn adjustable(self) -> bool {
        self.kind & TYPE_ADJUSTABLE != 0 && self.time < self.maximum_time
    }
}

pub(super) fn rewrite_layer(output: &mut Vec<u8>, layer_start: usize, state: &mut State) {
    let layer = output.split_off(layer_start);
    let mut lines = parse::layer(&layer, state);
    slowdown::apply(&mut lines, state.config);
    rewrite::append(output, &layer, &mut lines);
}
