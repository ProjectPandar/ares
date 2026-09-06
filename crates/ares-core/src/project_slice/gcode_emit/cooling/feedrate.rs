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

pub(super) fn rewrite_layer(output: &mut Vec<u8>, layer_start: usize, state: &mut State) -> f32 {
    let layer = output.split_off(layer_start);
    if let Ok(path) = std::env::var("ARES_DUMP_PRECOOLING") {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = file.write_all(b"=== LAYER ===\n");
            let _ = file.write_all(&layer);
        }
    }
    let mut lines = parse::layer(&layer, state);
    let layer_time = slowdown::apply(&mut lines, state.config);
    if let Ok(path) = std::env::var("ARES_DUMP_PRECOOLING") {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = file.write_all(b"=== SLOWDOWN ===\n");
            for line in &lines {
                let _ = writeln!(
                    file,
                    "kind={:x} len={:.4} feed={:.4} time={:.5} slowed={} start={} end={}",
                    line.kind,
                    line.length,
                    line.feedrate,
                    line.time,
                    line.slowed,
                    line.start,
                    line.end
                );
            }
        }
    }
    let pre_append = output.len();
    rewrite::append(output, &layer, &mut lines);
    if let Ok(path) = std::env::var("ARES_DUMP_PRECOOLING") {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = file.write_all(b"=== POSTCOOLING ===\n");
            let _ = file.write_all(&output[pre_append..]);
        }
    }
    layer_time
}
