//! Active belt prism exports from `Print::_plan_belt_purge` in `BeltPurge.cpp`.

#[path = "active_purge/fixture.rs"]
mod fixture;

use crate::{OrcaRunner, runner::native_api_cli::NativeCase};

const MODEL: &str = "cube10.stl";
const PRISM: &str = "purge-prism";
const COMPLETION: &str = "; PURGING FINISHED";

#[test]
fn directional_flush_matrix_selects_prism_fill_by_swap_direction() {
    compare(
        "belt-prism-directional-",
        (["6", "10"], true, "2"),
        |output| {
            output.check_swaps([(1.42664, false), (2.17857, true)]);
        },
    );
}

#[test]
fn prime_volume_can_finish_purging_without_prism_fill() {
    compare("belt-prism-prime-", (["40", "64"], false, "2"), |output| {
        output.check_swaps([(1.42664, false), (1.46069, false)]);
    });
}

#[test]
fn oversized_flush_request_uses_available_prism_fill_without_a_classic_tower() {
    compare(
        "belt-prism-oversized-",
        (["40", "64"], true, "2"),
        |output| {
            output.check_swaps([(1.97240, true), (2.17857, true)]);
        },
    );
}

#[test]
fn active_prism_aligns_but_keeps_its_layers_and_fills_without_a_swap() {
    compare("belt-prism-no-swap-", (["6", "10"], true, "1"), |output| {
        assert!(output.swaps.is_empty());
        assert_eq!(output.tools, [0]);
        assert!(!output.has_completion);
        assert_eq!(output.for_object(PRISM).last().unwrap().layer, "86.1675");
        assert!(
            output
                .for_object(PRISM)
                .any(|motion| motion.role.contains("infill"))
        );
    });
}

fn compare(
    label: &str,
    (matrix, use_matrix, solid_tool): ([&str; 2], bool, &str),
    check: impl FnOnce(Witness<'_>),
) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project = fixture::project(matrix, use_matrix, solid_tool);
    let native = NativeCase::slice_live(&runner, &project, label);
    let output = Witness::parse(&native.native);
    assert!(!native.native.contains(";TYPE:Prime tower\n"));
    assert!(!native.native.contains("; WIPE_TOWER_START\n"));
    let first_prism = output.for_object(PRISM).next().unwrap();
    assert_eq!(first_prism.layer, "77.9675");
    assert_eq!(
        output.for_object(MODEL).next().unwrap().layer,
        first_prism.layer
    );
    assert!(
        first_prism
            .line
            .split_ascii_whitespace()
            .any(|word| word == "Z110.263")
    );
    check(output);
    native.compare(ares_core::GenerationMetadata::default());
}

struct Motion<'a> {
    object: &'a str,
    role: &'a str,
    layer: &'a str,
    tool: u32,
    after_completion: bool,
    extrusion: f64,
    line: &'a str,
}

#[derive(Default)]
struct Witness<'a> {
    tools: Vec<u32>,
    swaps: Vec<(&'a str, &'a str)>,
    has_completion: bool,
    motions: Vec<Motion<'a>>,
}

impl<'a> Witness<'a> {
    fn parse(gcode: &'a str) -> Self {
        let mut result = Self::default();
        let (mut layer, mut role, mut object) = ("", "", None);
        let mut after_completion = false;
        for line in gcode.lines() {
            if let Some(z) = line.strip_prefix(";Z:") {
                layer = z;
                after_completion = false;
            } else if let Some(swap) = line.strip_prefix("; swap ") {
                result.swaps.push((layer, swap));
                after_completion = false;
            } else if let Some(tool) = line.strip_prefix('T').and_then(|text| text.parse().ok()) {
                result.tools.push(tool);
            } else if let Some(start) = line.strip_prefix("; printing object ") {
                object = start.split_once(" id:").map(|(name, _)| name);
            } else if line.starts_with("; stop printing object ") {
                object = None;
            } else if let Some(kind) = line.strip_prefix(";TYPE:") {
                role = kind;
            } else if line == COMPLETION {
                result.has_completion = true;
                after_completion = true;
            } else if let (Some(object), Some(&tool), Some(extrusion)) =
                (object, result.tools.last(), positive_motion_e(line))
            {
                result.motions.push(Motion {
                    object,
                    role,
                    layer,
                    tool,
                    after_completion,
                    extrusion,
                    line,
                });
            }
        }
        result
    }

    fn for_object(&self, name: &str) -> impl DoubleEndedIterator<Item = &Motion<'a>> + Clone {
        self.motions
            .iter()
            .filter(move |motion| motion.object == name)
    }

    fn check_swaps(&self, budgets: [(f64, bool); 2]) {
        assert_eq!(
            self.swaps[..3],
            [
                ("", "-1 -> 0"),
                ("79.1675", "0 -> 1"),
                ("79.3675", "1 -> 0")
            ]
        );
        assert_eq!(self.tools[..3], [0, 1, 0]);
        assert_eq!(self.swaps.last().copied(), Some(("84.9675", "1 -> 0")));
        assert_eq!(self.for_object(PRISM).last().unwrap().layer, "84.9675");
        assert_eq!(self.for_object(MODEL).last().unwrap().layer, "86.1675");
        assert!(
            self.for_object(PRISM)
                .filter(|motion| motion.layer < "79.1675")
                .all(|motion| !motion.role.contains("infill"))
        );
        for ((height, tool), (e, keeps_fill)) in
            [("79.1675", 1), ("79.3675", 0)].into_iter().zip(budgets)
        {
            let prism_wipe = self.for_object(PRISM).filter(|motion| {
                motion.layer == height && motion.tool == tool && !motion.after_completion
            });
            let sum: f64 = prism_wipe.clone().map(|motion| motion.extrusion).sum();
            assert!(
                (sum - e).abs() < 0.000005,
                "{height} T{tool}: expected E{e}, got E{sum}"
            );
            assert_eq!(
                prism_wipe
                    .clone()
                    .any(|motion| motion.role.contains("infill")),
                keeps_fill
            );
            assert!(prism_wipe.clone().any(|motion| motion.role == "Outer wall"));
        }
        assert!(self.for_object(PRISM).any(|motion| {
            motion.layer == "79.1675"
                && motion.tool == 1
                && !motion.after_completion
                && motion.line.starts_with("G1 X65.675 Y1.947 Z111.96 E")
        }));
        let model_fill = self
            .for_object(MODEL)
            .find(|motion| {
                motion.layer == "79.1675"
                    && motion.tool == 1
                    && motion.role == "Internal solid infill"
            })
            .expect("model infill on the new tool");
        assert!(model_fill.after_completion);
        assert!(model_fill.line.starts_with("G1 X101.082 Y1.182 Z111.96 E"));
    }
}

fn positive_motion_e(line: &str) -> Option<f64> {
    let words = line
        .strip_prefix("G1 ")?
        .split_ascii_whitespace()
        .take_while(|word| !word.starts_with(';'));
    if !words.clone().any(|word| word.starts_with(['X', 'Y', 'Z'])) {
        return None;
    }
    words
        .filter_map(|word| word.strip_prefix('E'))
        .filter_map(|e| e.parse::<f64>().ok())
        .find(|&e| e > 0.0)
}
