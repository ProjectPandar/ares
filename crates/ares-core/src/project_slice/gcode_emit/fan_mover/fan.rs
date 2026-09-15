//! Fan command processing (`FanMover::process_fan_command`) — the
//! M106/M107 handling, kickstart insertion, and gradient back-fill.

use crate::ExtrusionRole;

use super::helpers::{LineMotion, split_line};
use super::helpers::{fan_pwm, set_fan_line};
use super::{BufferData, FanMover, Kickstart};

impl FanMover {
    pub(super) fn process_fan_command(&mut self, raw: &str, fan_speed: i16, time: &mut f32) {
        if self.back_buffer_fan_speed >= fan_speed {
            return;
        }
        if self.nb_seconds_delay > 0.0
            && (!self.only_overhangs || self.current_role == ExtrusionRole::OverhangPerimeter)
        {
            *time = -1.0;
            if self.kickstart > 0.0 && fan_speed > self.front_buffer_fan_speed {
                self.current_kickstart = None;
                self.remove_slow_fan(fan_speed, self.buffer_time_size + 1.0);
                self.remove_slow_fan(255, self.kickstart);
                if !self.buffer.is_empty()
                    && (self.buffer_time_size - self.buffer[0].time * 0.1) > self.nb_seconds_delay
                {
                    self.print_in_middle(
                        0,
                        self.buffer_time_size - self.nb_seconds_delay,
                        &set_fan_line(100),
                    );
                } else {
                    self.output.push_str(&set_fan_line(100));
                }
                let kickstart_duration =
                    self.kickstart * f32::from(fan_speed - self.front_buffer_fan_speed) / 100.0;
                let mut time_count = kickstart_duration;
                let mut index = 0;
                while index < self.buffer.len() && time_count > 0.0 {
                    time_count -= self.buffer[index].time;
                    if time_count < 0.0 {
                        let data = BufferData::new(raw.to_owned(), 0.0, fan_speed, true);
                        self.put_in_middle(index, self.buffer[index].time + time_count, data);
                        break;
                    }
                    index += 1;
                }
                if time_count > 0.0 {
                    self.current_kickstart = Some(Kickstart {
                        fan_speed,
                        time: time_count,
                    });
                    self.current_kickstart_raw = raw.to_owned();
                }
                self.front_buffer_fan_speed = fan_speed;
            } else {
                self.remove_slow_fan(fan_speed, self.buffer_time_size + 1.0);
                if !self.buffer.is_empty()
                    && (self.buffer_time_size - self.buffer[0].time * 0.1) > self.nb_seconds_delay
                {
                    self.print_in_middle(0, self.buffer_time_size - self.nb_seconds_delay, raw);
                } else {
                    self.output.push_str(raw);
                    self.output.push('\n');
                }
                self.front_buffer_fan_speed = fan_speed;
            }
        } else if self.kickstart <= 0.0 {
            // Nothing to do — printed in the buffer as other lines are.
        } else if self.current_kickstart.is_some() {
            if let Some(kick) = &mut self.current_kickstart {
                if self.back_buffer_fan_speed >= fan_speed {
                    self.current_kickstart = None;
                } else {
                    let kickstart_duration =
                        self.kickstart * f32::from(fan_speed - self.back_buffer_fan_speed) / 100.0;
                    kick.fan_speed = fan_speed;
                    kick.time += kickstart_duration;
                    self.current_kickstart = Some(*kick);
                    self.current_kickstart_raw = raw.to_owned();
                    *time = -1.0;
                }
            }
        } else if self.back_buffer_fan_speed < fan_speed - 10 {
            *time = -1.0;
            let kickstart_duration =
                self.kickstart * f32::from(fan_speed - self.back_buffer_fan_speed) / 100.0;
            self.push_buffer(
                BufferData::new(set_fan_line(100), 0.0, fan_speed, true),
                &LineMotion::default(),
                self.position,
                self.e_position,
            );
            self.current_kickstart = Some(Kickstart {
                fan_speed,
                time: kickstart_duration,
            });
            self.current_kickstart_raw = raw.to_owned();
        }
    }

    pub(super) fn push_buffer(
        &mut self,
        data: BufferData,
        motion: &LineMotion,
        start: [f32; 3],
        e_start: f32,
    ) {
        self.buffer_time_size += data.time;
        let mut data = data;
        // FanMover.cpp:444-449 stores the START coordinate (reader position
        // before the move) plus the delta; the split math `x + dx*percent`
        // interpolates within the segment.
        if motion.x.is_some() {
            data.x = start[0];
            data.dx = motion.dx;
        }
        if motion.y.is_some() {
            data.y = start[1];
            data.dy = motion.dy;
        }
        if motion.z.is_some() {
            data.z = start[2];
            data.dz = motion.dz;
        }
        if motion.e.is_some() {
            data.e = e_start;
            data.de = motion.de;
        }
        self.buffer.push(data);
    }

    /// `_put_in_middle_G1` (FanMover.cpp:124-140): split the buffered line
    /// at `nb_sec` from its start, inserting the fan line between.
    pub(super) fn put_in_middle(&mut self, index: usize, nb_sec: f32, line: BufferData) {
        let item_time = self.buffer[index].time;
        if nb_sec > item_time * 0.9 {
            self.buffer.insert(index + 1, line);
        } else if nb_sec < item_time * 0.1 || item_time == 0.0 {
            self.buffer.insert(index, line);
        } else {
            let percent = nb_sec / item_time;
            let (before_raw, after_raw) =
                split_line(&mut self.buffer[index], percent, self.relative_e);
            let before = BufferData::new(before_raw, nb_sec, -1, false);
            let after_time = item_time - nb_sec;
            // FanMover.cpp:131-142: advance the remaining item's start
            // coordinates by the consumed prefix delta.
            let (dx, dy, dz, de) = (
                self.buffer[index].dx,
                self.buffer[index].dy,
                self.buffer[index].dz,
                self.buffer[index].de,
            );
            self.buffer[index].x += dx * percent;
            self.buffer[index].y += dy * percent;
            self.buffer[index].z += dz * percent;
            if !self.relative_e {
                self.buffer[index].e += de * percent;
            }
            self.buffer[index].raw = after_raw;
            self.buffer[index].time = after_time;
            self.buffer[index].dx = dx * (1.0 - percent);
            self.buffer[index].dy = dy * (1.0 - percent);
            self.buffer[index].dz = dz * (1.0 - percent);
            self.buffer[index].de = de * (1.0 - percent);
            self.buffer.insert(index, line);
            self.buffer.insert(index, before);
            self.buffer_time_size += 0.0;
        }
    }

    /// `_print_in_middle_G1` (FanMover.cpp:171-212): flush the front line
    /// (optionally split) with the fan command inside.
    fn print_in_middle(&mut self, index: usize, nb_sec: f32, fan_line: &str) {
        let item = self.buffer.remove(index);
        self.buffer_time_size -= item.time;
        if nb_sec < item.time * 0.1 {
            self.output.push_str(&item.raw);
            self.output.push('\n');
            self.output.push_str(fan_line);
            if !fan_line.ends_with('\n') {
                self.output.push('\n');
            }
        } else if nb_sec > item.time * 0.9 || !item.raw.starts_with("G1 ") {
            self.output.push_str(fan_line);
            if !fan_line.ends_with('\n') {
                self.output.push('\n');
            }
            self.output.push_str(&item.raw);
            self.output.push('\n');
        } else {
            let percent = nb_sec / item.time;
            let mut item = item;
            let (before, after) = split_line(&mut item, percent, self.relative_e);
            self.output.push_str(&before);
            self.output.push('\n');
            self.output.push_str(fan_line);
            if !fan_line.ends_with('\n') {
                self.output.push('\n');
            }
            self.output.push_str(&after);
            self.output.push('\n');
        }
    }

    /// `_remove_slow_fan` (FanMover.cpp:214-227).
    fn remove_slow_fan(&mut self, min_speed: i16, mut past_sec: f32) {
        let mut index = 0;
        while index < self.buffer.len() && past_sec > 0.0 {
            past_sec -= self.buffer[index].time;
            if self.buffer[index].fan_speed >= 0 && self.buffer[index].fan_speed < min_speed {
                let removed = self.buffer.remove(index);
                self.buffer_time_size -= removed.time;
            } else {
                index += 1;
            }
        }
    }
}
