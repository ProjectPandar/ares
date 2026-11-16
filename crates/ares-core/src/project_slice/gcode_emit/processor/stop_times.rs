//! Normal `TimeMachine::stop_times` and M73 C postprocessing.
//! GCodeProcessor.cpp:576-579,1098-1101,1260-1293,5715.

use super::time::minutes;

struct StopTime {
    g1_id: usize,
    elapsed: f32,
}

#[derive(Default)]
pub(super) struct StopTimes {
    stops: Vec<StopTime>,
}

impl StopTimes {
    pub(super) fn register(&mut self, g1_id: usize) {
        self.stops.push(StopTime {
            g1_id,
            elapsed: 0.0,
        });
    }

    // Observe the existing eligible cache snapshot; do not calculate time.
    // lower_bound updates only the first stop with a repeated G1 ID (:576).
    pub(super) fn record(&mut self, id: usize, elapsed: f32) {
        let index = self.stops.partition_point(|stop| stop.g1_id < id);
        if let Some(stop) = self.stops.get_mut(index)
            && stop.g1_id == id
        {
            stop.elapsed = elapsed;
        }
    }

    pub(super) fn exporter(&self, total: f64) -> StopExport<'_> {
        StopExport {
            stops: &self.stops,
            last: minutes(total),
        }
    }
}

pub(super) struct StopExport<'a> {
    stops: &'a [StopTime],
    last: u64,
}

impl StopExport<'_> {
    pub(super) fn initial(&mut self) -> Option<String> {
        let first = self.stops.first()?;
        self.last = minutes(f64::from(first.elapsed));
        Some(format!("M73 C{}\n", self.last))
    }

    pub(super) fn progress(&mut self, elapsed: f32, next: Option<f32>) -> Option<String> {
        let index = self.upper_bound(elapsed);
        let stop = self.stops.get(index)?;
        let remaining = stop.elapsed - elapsed;
        let to_export = minutes(f64::from(remaining));
        if self.last == to_export {
            return None;
        }
        let line = if to_export > 0 {
            format!("M73 C{to_export}\n")
        } else {
            let is_last = match next {
                None => true,
                Some(next) => {
                    // GCodeProcessor.cpp:1281-1283 compares the two-decimal strings
                    // after parsing; compare their signs/zero forms without parsing.
                    let remaining = fraction(remaining);
                    self.upper_bound(next) != index
                        || remaining != "0.00"
                            && remaining != "-0.00"
                            && !remaining.starts_with('-')
                            && matches!(fraction(stop.elapsed - next).as_str(), "0.00" | "-0.00")
                }
            };
            if !is_last {
                return None;
            }
            if index + 1 == self.stops.len() {
                format!("M73 C{to_export}\n")
            } else {
                format!("M73 C{}\n", fraction(remaining))
            }
        };
        self.last = to_export;
        Some(line)
    }

    // Preserve std::upper_bound's traversal: a SYNC-flushed registration may
    // retain zero even after earlier positive stops. Do not sort/fill it.
    fn upper_bound(&self, elapsed: f32) -> usize {
        let mut first = 0;
        let mut count = self.stops.len();
        while count != 0 {
            let step = count / 2;
            let middle = first + step;
            if elapsed < self.stops[middle].elapsed {
                count = step;
            } else {
                first = middle + 1;
                count -= step + 1;
            }
        }
        first
    }
}

fn fraction(seconds: f32) -> String {
    format!("{:.2}", seconds / 60.0_f32)
}
