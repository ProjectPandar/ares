//! Port of `GCode/FanMover.cpp` — the fan-speedup post-pass that delays
//! fan-up commands into the recent past (a time-buffered g-code rewriter).
//! Active only when `fan_speedup_time != 0 || fan_kickstart > 0`
//! (`GCode.cpp:3731-3740`).

mod fan;
mod helpers;
mod motion;

use crate::{ExtrusionRole, GCodeFlavor};

use helpers::set_fan_line;

use helpers::{LineMotion, parse_motion, read_fan_speed, role_from_str};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct BufferData {
    raw: String,
    time: f32,
    fan_speed: i16,
    is_kickstart: bool,
    /// End coordinates / deltas of a G0/G1 line (`-1` when the word is
    /// absent, mirroring the reader's NaN-free sentinel usage via `has`).
    x: f32,
    y: f32,
    z: f32,
    e: f32,
    dx: f32,
    dy: f32,
    dz: f32,
    de: f32,
}

impl BufferData {
    pub(super) fn new(raw: String, time: f32, fan_speed: i16, is_kickstart: bool) -> Self {
        Self {
            raw,
            time,
            fan_speed,
            is_kickstart,
            x: -1.0,
            y: -1.0,
            z: -1.0,
            e: -1.0,
            dx: 0.0,
            dy: 0.0,
            dz: 0.0,
            de: 0.0,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct Kickstart {
    fan_speed: i16,
    time: f32,
}

pub(crate) struct FanMover {
    nb_seconds_delay: f32,
    only_overhangs: bool,
    kickstart: f32,
    relative_e: bool,
    flavor: GCodeFlavor,
    gcode_comments: bool,
    buffer: Vec<BufferData>,
    buffer_time_size: f64,
    front_buffer_fan_speed: i16,
    back_buffer_fan_speed: i16,
    current_kickstart: Option<Kickstart>,
    current_kickstart_raw: String,
    current_speed: f64,
    is_custom_gcode: bool,
    current_role: ExtrusionRole,
    output: String,
    /// Tracked XYZ position (for move distances) and E (for absolute-E
    /// deltas), mirroring the upstream GCodeReader state.
    position: [f32; 3],
    e_position: f32,
}

impl FanMover {
    pub(crate) fn new(
        fan_speedup_time: f64,
        fan_kickstart: f64,
        only_overhangs: bool,
        relative_e: bool,
        (flavor, gcode_comments): (GCodeFlavor, bool),
    ) -> Self {
        let delay = if fan_speedup_time > 0.0 {
            (fan_speedup_time as f32).max(0.01)
        } else {
            0.0
        };
        Self {
            nb_seconds_delay: delay,
            only_overhangs,
            kickstart: fan_kickstart as f32,
            relative_e,
            flavor,
            gcode_comments,
            buffer: Vec::new(),
            buffer_time_size: 0.0,
            // FanMover.hpp:51-52: both buffer fan speeds start at 0 (the
            // initial state IS "fan off"), so a chunk-leading `M106 S0`
            // with speed == front speed is suppressed at flush.
            front_buffer_fan_speed: 0,
            back_buffer_fan_speed: 0,
            current_kickstart: None,
            current_kickstart_raw: String::new(),
            current_speed: 1000.0 / 60.0,
            is_custom_gcode: false,
            current_role: ExtrusionRole::None,
            output: String::new(),
            position: [0.0; 3],
            e_position: 0.0,
        }
    }

    pub(crate) fn process_gcode(&mut self, gcode: &str, flush: bool) -> String {
        self.output.clear();
        self.buffer_time_size = 0.0;
        for data in &self.buffer {
            self.buffer_time_size += f64::from(data.time);
        }
        for line in gcode.lines() {
            self.process_line(line);
        }
        if flush {
            self.flush_remaining();
        }
        std::mem::take(&mut self.output)
    }

    fn process_line(&mut self, raw: &str) {
        let mut need_flush = false;
        let mut time: f64 = 0.0;
        let mut fan_speed: i16 = -1;
        let mut motion = LineMotion::default();
        let code = raw.split(';').next().unwrap_or_default().trim();
        let command = code.split_whitespace().next().unwrap_or_default();
        if command.len() > 1 {
            motion = parse_motion(code);
            if motion.has_f {
                self.current_speed = f64::from(motion.f_mm_s);
            }
            // GCodeReader only tracks position for G0/G1/G2/G3/G92 —
            // axis words on M/T commands (e.g. `M205 X12 Y12`) must not
            // teleport the tracked position.
            if !command.starts_with('G') {
                motion.x = None;
                motion.y = None;
                motion.z = None;
                motion.e = None;
            }
            match command.as_bytes()[0] {
                b'T' => need_flush = true,
                b'G' => time = self.process_g_command(command, &mut motion),
                b'M' => (fan_speed, need_flush) = self.process_m_command(raw, &mut time),
                _ => {}
            }
        } else if let Some(comment) = raw.strip_prefix(';') {
            if let Some(role) = comment.strip_prefix("TYPE:") {
                self.current_role = role_from_str(role);
            }
            if raw.contains("; custom gcode") {
                self.is_custom_gcode = !raw.contains("; custom gcode end");
            }
        }

        if time >= 0.0 {
            let start = self.position;
            let e_start = self.e_position;
            self.track_motion(&mut motion);
            self.push_buffer(
                BufferData::new(raw.to_owned(), time as f32, fan_speed, false),
                &motion,
                start,
                e_start,
            );
            self.advance_kickstart(time);
        }

        while !self.buffer.is_empty()
            && (need_flush
                || self.nb_seconds_delay <= f32::EPSILON
                || self.buffer_time_size - f64::from(self.buffer[0].time)
                    > f64::from(self.nb_seconds_delay - f32::EPSILON))
        {
            let front = self.buffer.remove(0);
            self.buffer_time_size -= f64::from(front.time);
            self.emit_front(front);
        }
    }

    fn flush_remaining(&mut self) {
        while let Some(front) = self.buffer.first().cloned() {
            if front.fan_speed >= 0 {
                self.front_buffer_fan_speed = front.fan_speed;
            }
            self.output.push_str(&front.raw);
            self.output.push('\n');
            self.buffer.remove(0);
        }
    }

    fn process_m_command(&mut self, raw: &str, time: &mut f64) -> (i16, bool) {
        let mut need_flush = false;
        let fan_speed = read_fan_speed(raw, self.flavor);
        if fan_speed >= 0 {
            self.current_kickstart = None;
            if !self.is_custom_gcode {
                self.process_fan_command(raw, fan_speed, time);
            } else {
                need_flush = true;
            }
            self.back_buffer_fan_speed = fan_speed;
        }
        (fan_speed, need_flush)
    }

    fn advance_kickstart(&mut self, time: f64) {
        if let Some(mut kick) = self.current_kickstart
            && time > 0.0
        {
            kick.time = (f64::from(kick.time) - time) as f32;
            if kick.time < 0.0 {
                let split = (time + f64::from(kick.time)) as f32;
                let data = BufferData::new(
                    std::mem::take(&mut self.current_kickstart_raw),
                    0.0,
                    kick.fan_speed,
                    true,
                );
                self.put_in_middle(self.buffer.len() - 1, split, data);
            }
            self.current_kickstart = (kick.time >= 0.0).then_some(kick);
        }
    }

    fn emit_front(&mut self, front: BufferData) {
        if front.fan_speed >= 0
            && front.fan_speed == self.front_buffer_fan_speed
            && !front.is_kickstart
        {
            return;
        }
        if front.is_kickstart && front.fan_speed < self.front_buffer_fan_speed {
            self.output
                .push_str(&set_fan_line(front.fan_speed, self.gcode_comments));
            self.front_buffer_fan_speed = front.fan_speed;
        } else {
            self.output.push_str(&front.raw);
            self.output.push('\n');
            if front.fan_speed >= 0 {
                self.front_buffer_fan_speed = front.fan_speed;
            }
        }
    }
}
