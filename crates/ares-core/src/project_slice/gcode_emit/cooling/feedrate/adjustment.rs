//! `CoolingBuffer.cpp::PerExtruderAdjustments` (107–253).
use super::{Config, CoolingLine};
use crate::geometry::fixed_gcc_sort_by;

pub(super) struct Adjustment {
    pub(super) config: Config,
    pub(super) lines: Vec<CoolingLine>,
    pub(super) adjustable: usize,
    pub(super) total: f32,
    pub(super) maximum: f32,
    pub(super) begin: usize,
    pub(super) end: usize,
}

impl Adjustment {
    pub(super) const fn new(config: Config) -> Self {
        Self {
            config,
            lines: Vec::new(),
            adjustable: 0,
            total: 0.0,
            maximum: 0.0,
            begin: 0,
            end: 0,
        }
    }

    pub(super) fn elapsed(&self) -> f32 {
        self.lines
            .iter()
            .fold(0.0_f32, |total, line| total + line.time)
    }

    pub(super) fn maximum_time(&self) -> f32 {
        let mut total = 0.0;
        for line in &self.lines {
            if !line.adjustable() {
                total += line.time;
                continue;
            }
            if line.maximum_time == f32::MAX {
                return f32::MAX;
            }
            total += line.maximum_time;
        }
        total
    }

    pub(super) fn sort(&mut self) {
        fixed_gcc_sort_by(&mut self.lines, |left, right| {
            if left.adjustable() == right.adjustable() {
                left.feedrate > right.feedrate
            } else {
                left.adjustable()
            }
        });
        self.adjustable = self
            .lines
            .iter()
            .take_while(|line| line.adjustable())
            .count();
    }

    pub(super) fn slow_to_minimum(&mut self) {
        for line in self.lines.iter_mut().filter(|line| line.adjustable()) {
            line.slowed = true;
            line.time = line.maximum_time;
            if line.time > 0.0 {
                line.feedrate = line.length / line.time;
            }
        }
        self.total = self.elapsed();
    }

    pub(super) fn adjustable_time(&self) -> f32 {
        self.lines
            .iter()
            .filter(|line| line.adjustable())
            .fold(0.0_f32, |total, line| total + line.time)
    }

    pub(super) fn proportional(&mut self, factor: f32) {
        for line in self.lines.iter_mut().filter(|line| line.adjustable()) {
            line.slowed = true;
            line.time = line.maximum_time.min(line.time * factor);
            if line.time > 0.0 {
                line.feedrate = line.length / line.time;
            }
        }
        self.total = self.elapsed();
    }

    pub(super) fn stretch_to(&self, feedrate: f32) -> f32 {
        self.lines[..self.adjustable]
            .iter()
            .filter(|line| line.feedrate > feedrate)
            .fold(0.0_f32, |total, line| {
                total + line.time * (line.feedrate / feedrate - 1.0)
            })
    }

    pub(super) fn slow_to_feedrate(&mut self, feedrate: f32) {
        let mut total = 0.0;
        for line in &mut self.lines[..self.adjustable] {
            if line.feedrate > feedrate {
                line.time *= (line.feedrate / feedrate).max(1.0);
                line.feedrate = feedrate;
                line.slowed = true;
            }
            total += line.time;
        }
        // Source stores only the adjustable prefix here (cpp:218–228).
        self.total = total;
    }
}
