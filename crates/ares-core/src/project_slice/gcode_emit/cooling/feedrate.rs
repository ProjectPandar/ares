mod adjustment;
mod parse;
mod range;
mod rewrite;
mod slowdown;

use adjustment::Adjustment;

const TYPE_SET_TOOL: u32 = 1;
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
    /// Preserve the `;_EXTRUDE_SET_SPEED` / `;_EXTRUDE_END` marker
    /// scaffolding for the PressureEqualizer pass (upstream keeps the
    /// markers through the cooling buffer; only the processor strips
    /// them after the equalizer consumed them).
    pub(super) keep_markers: bool,
}

pub(super) struct State {
    configs: Vec<Config>,
    tool_map: Vec<usize>,
    current_tool: usize,
    toolchange_prefix: &'static str,
    position: [f32; 7],
}

impl State {
    pub(super) fn new(
        configs: Vec<(usize, Config)>,
        travel_speed: f64,
        toolchange_prefix: &'static str,
        current_tool: usize,
    ) -> Self {
        let mut position = [0.0; 7];
        position[4] = travel_speed as f32;
        let mut tool_map = vec![0; configs.last().unwrap().0 + 1];
        for (index, (tool, _)) in configs.iter().enumerate() {
            tool_map[*tool] = index;
        }
        Self {
            configs: configs.into_iter().map(|(_, config)| config).collect(),
            tool_map,
            current_tool,
            toolchange_prefix,
            position,
        }
    }

    /// `CoolingBuffer`'s `m_current_extruder`: the last `T<tool>` seen while
    /// parsing layer G-code (`CoolingBuffer.cpp:840-850`), retained across
    /// layers. Fan options index this logical filament id.
    pub(super) fn current_tool(&self) -> usize {
        self.current_tool
    }

    pub(super) const fn toolchange_prefix(&self) -> &'static str {
        self.toolchange_prefix
    }

    /// Accepted tool id bound, matching `parse::layer`'s
    /// `tool < tool_map.len()` gate.
    pub(super) fn tool_bound(&self) -> usize {
        self.tool_map.len()
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
    let (mut adjustments, current_tool) = parse::layer(&layer, state);
    let layer_time = slowdown::apply(&mut adjustments);
    let mut lines = adjustments
        .iter()
        .flat_map(|adjustment| &adjustment.lines)
        .collect::<Vec<_>>();
    rewrite::append(output, &layer, &mut lines, state.configs[0].keep_markers);
    state.current_tool = current_tool;
    layer_time
}
