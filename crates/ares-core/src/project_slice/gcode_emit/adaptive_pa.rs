//! `AdaptivePAProcessor`, after cooling and fan movement.

mod model;
mod number;
mod pchip;
mod scan;

use super::PreparedPostClassicTraversal;
use crate::options::ProjectSettings;
use regex::Regex;
use std::fmt::Write;
use std::sync::LazyLock;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Options {
    pub(super) enabled: bool,
    pub(super) overhangs: bool,
}

impl Options {
    pub(super) fn new(full: &ProjectSettings, tool: usize) -> Self {
        Self {
            enabled: enabled(full, tool),
            overhangs: at(
                &full.filament.gcode.adaptive_pressure_advance_overhangs.0,
                tool,
            )
            .0,
        }
    }
}

pub(super) fn enabled(full: &ProjectSettings, tool: usize) -> bool {
    let filament = &full.filament.gcode;
    at(&filament.enable_pressure_advance.0, tool).0
        && at(&filament.adaptive_pressure_advance.0, tool).0
}

// `ConfigOptionVector::get_at` broadcasts the first value for a short vector.
fn at<T>(values: &[T], tool: usize) -> &T {
    values.get(tool).unwrap_or(&values[0])
}

pub(super) fn process(
    layers: Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    state: &super::motion::EmitState,
) -> Vec<u8> {
    let full = &traversal.resolved.views.full;
    if !state
        .extruders
        .ids()
        .iter()
        .any(|&tool| enabled(full, tool))
        || full.process.print.spiral_mode.0
    {
        return layers;
    }
    let filament = &full.filament.gcode;
    let mut processor = Processor {
        models: state
            .extruders
            .ids()
            .iter()
            .copied()
            .filter(|&tool| enabled(full, tool))
            .map(|tool| {
                (
                    tool,
                    model::Model::parse(at::<String>(
                        &filament.adaptive_pressure_advance_model.0,
                        tool,
                    )),
                )
            })
            .collect(),
        full,
        is_bbl: super::tags::Tags::of(traversal).is_bbl(),
        previous: 0.0,
        current_speed: 0.0,
        modal_speed: 0.0,
        last_tool: None,
    };
    let text = String::from_utf8(layers).expect("generated G-code is UTF-8");
    let mut result = String::with_capacity(text.len());
    let marker = format!("{}\n", super::tags::Tags::of(traversal).layer_change());
    let mut chunk_start = None;
    let mut resets = state.adaptive_pa_resets.iter().peekable();
    let mut layer = 0;
    for (offset, _) in text.match_indices(&marker) {
        if offset != 0 && text.as_bytes()[offset - 1] != b'\n' {
            continue;
        }
        if let Some(start) = chunk_start {
            processor.reset(layer, &mut resets);
            processor.layer(&text[start..offset], &mut result);
            layer += 1;
        } else {
            result.push_str(&text[..offset]);
        }
        chunk_start = Some(offset);
    }
    if let Some(start) = chunk_start {
        processor.reset(layer, &mut resets);
        processor.layer(&text[start..], &mut result);
    } else {
        result.push_str(&text);
    }
    result.into_bytes()
}

struct Processor<'a> {
    models: std::collections::BTreeMap<usize, Option<model::Model>>,
    full: &'a ProjectSettings,
    is_bbl: bool,
    previous: f64,
    current_speed: f64,
    modal_speed: f64,
    last_tool: Option<usize>,
}

impl Processor<'_> {
    fn reset<'a>(
        &mut self,
        layer: usize,
        resets: &mut std::iter::Peekable<impl Iterator<Item = &'a (usize, f64)>>,
    ) {
        while resets.peek().is_some_and(|&&(index, _)| index <= layer) {
            self.previous = resets.next().expect("peeked reset").1;
        }
    }

    fn layer(&mut self, text: &str, output: &mut String) {
        let lines: Vec<_> = text.split_terminator('\n').collect();
        let mut wiping = false;
        for (index, line) in lines.iter().enumerate() {
            if line.contains("WIPE_START") {
                wiping = true;
            }
            if !wiping && let Some(speed) = scan::feedrate(line) {
                self.current_speed = speed;
            }
            if let Some(speed) = scan::move_feedrate(line) {
                self.modal_speed = speed;
            }
            if line.contains("WIPE_END") {
                wiping = false;
            }
            if !line.starts_with("; PA_CHANGE") {
                writeln!(output, "{line}").unwrap();
                continue;
            }
            self.tag(line, &lines[index + 1..], output);
        }
    }

    fn tag(&mut self, line: &str, following: &[&str], output: &mut String) {
        static TAG: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(
                r"; PA_CHANGE:T(\d+) MM3MM:([0-9]*\.[0-9]+) ACCEL:(\d+) BR:(\d+) RC:(\d+) OV:(\d+)",
            )
            .expect("upstream PA tag expression")
        });
        let Some(tag) = TAG.captures(line) else {
            return;
        };
        let tool = tag[1].parse::<usize>().expect("matched tool");
        let flow = tag[2].parse::<f64>().expect("matched flow");
        let acceleration = tag[3].parse::<f64>().expect("matched acceleration");
        let bridge = &tag[4] != "0";
        let overhang = &tag[6] != "0";
        let changed_tool = self.last_tool != Some(tool);
        self.last_tool = Some(tool);
        let speeds = scan::lookahead(following, self.current_speed, self.modal_speed);
        self.current_speed = speeds.current;
        let filament = &self.full.filament.gcode;
        let default = if at(&filament.enable_pressure_advance.0, tool).0 {
            at(&filament.pressure_advance.0, tool).0
        } else {
            0.0
        };
        let mut used_speed = 0.0;
        let mut predicted = default;
        let comments = self.full.process.print.gcode_comments.0;
        if let Some(Some(model)) = self.models.get(&tool) {
            used_speed = if !overhang {
                speeds.maximum.max(speeds.current)
            } else if speeds.current == 0.0 || speeds.next == 0.0 {
                speeds.current.max(speeds.next)
            } else {
                speeds.current.min(speeds.next)
            };
            predicted = model.predict(flow * used_speed, acceleration);
            if bridge && at(&filament.adaptive_pressure_advance_bridges.0, tool).0 > 1e-4 {
                predicted = at(&filament.adaptive_pressure_advance_bridges.0, tool).0;
            }
            if predicted < 0.0 && comments {
                output.push_str(
                    "; APA: Interpolation failed, using fallback pressure advance value\n",
                );
            }
            if predicted < 0.0 {
                predicted = default;
            }
        } else if comments {
            output.push_str(if self.models.contains_key(&tool) {
                "; APA: Interpolator setup failed, using default pressure advance\n"
            } else {
                "; APA: Tool doesnt have APA enabled\n"
            });
        }
        if comments {
            writeln!(output, "{line}").unwrap();
            if bridge && at(&filament.adaptive_pressure_advance_bridges.0, tool).0 > 1e-4 {
                output.push_str("; APA Model Override (bridge)\n");
            }
            writeln!(output, "; APA Current Speed: {:.6}\n; APA Next Speed: {:.6}\n; APA Max Next Speed: {:.6}\n; APA Speed Used: {used_speed:.6}\n; APA Flow rate: {:.6}\n; APA Prev PA: {:.6} New PA: {predicted:.6}",
                    speeds.current, speeds.next, speeds.maximum, flow * speeds.maximum, self.previous).unwrap();
        }
        if changed_tool || (predicted - self.previous).abs() > 1e-4 {
            output.push_str(&super::pressure_advance::command(
                predicted,
                self.full.printer.gcode.gcode_flavor,
                self.is_bbl,
            ));
            self.previous = predicted;
        }
    }
}
