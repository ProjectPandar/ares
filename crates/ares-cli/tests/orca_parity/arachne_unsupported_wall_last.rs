//! Public Native movement premises for beta Arachne's collection deferral call.

use crate::runner::native_stream_case;

struct Stroke<'a> {
    role: &'a str,
    points: Vec<(f64, f64)>,
}

impl Stroke<'_> {
    fn closed(&self) -> bool {
        self.points.len() >= 4 && self.points.first() == self.points.last()
    }

    fn footprint(&self) -> f64 {
        let (min, max) = self.points.iter().fold(
            (
                (f64::INFINITY, f64::INFINITY),
                (f64::NEG_INFINITY, f64::NEG_INFINITY),
            ),
            |(min, max), &(x, y)| ((min.0.min(x), min.1.min(y)), (max.0.max(x), max.1.max(y))),
        );
        (max.0 - min.0) * (max.1 - min.1)
    }

    fn infill(&self) -> bool {
        matches!(
            self.role,
            "Bridge"
                | "Internal Bridge"
                | "Internal solid infill"
                | "Bottom surface"
                | "Sparse infill"
        )
    }
}

fn word(code: &str, prefix: char) -> Option<f64> {
    code.split_whitespace()
        .find_map(|word| word.strip_prefix(prefix))
        .map(|value| value.parse().unwrap())
}

#[derive(Default)]
struct Observer<'a> {
    strokes: Vec<Stroke<'a>>,
    position: (f64, f64),
    role: &'a str,
    shoulder: bool,
    wiping: bool,
    contiguous: bool,
}

impl<'a> Observer<'a> {
    fn push(&mut self, line: &'a str) {
        if let Some(z) = line.strip_prefix(";Z:") {
            self.shoulder = z.parse::<f64>().unwrap() == 1.4;
            self.contiguous = false;
        }
        if let Some(role) = line.strip_prefix(";TYPE:") {
            self.role = role;
        }
        match line {
            ";WIPE_START" => self.wiping = true,
            ";WIPE_END" => self.wiping = false,
            _ => {}
        }
        let code = line.split(';').next().unwrap();
        if matches!(code.split_whitespace().next(), Some("G0" | "G1")) {
            self.motion(code);
        }
    }

    fn motion(&mut self, code: &str) {
        let before = self.position;
        let x = word(code, 'X');
        let y = word(code, 'Y');
        self.position = (x.unwrap_or(before.0), y.unwrap_or(before.1));
        if x.is_none() && y.is_none() {
            return;
        }
        if !self.shoulder || self.wiping || word(code, 'E').unwrap_or(0.0) <= 0.0 {
            self.contiguous = false;
            return;
        }
        if !self.contiguous
            || self
                .strokes
                .last()
                .is_none_or(|stroke| stroke.role != self.role)
        {
            self.strokes.push(Stroke {
                role: self.role,
                points: vec![before],
            });
        }
        self.strokes.last_mut().unwrap().points.push(self.position);
        self.contiguous = true;
    }
}

enum NativeOrder {
    InfillBeforeWalls,
    WallsBeforeInfill,
    AnchoredBeforeOverhang,
}

fn check_order(native: &str, order: &NativeOrder) {
    assert!(native.contains("; wall_generator = arachne\n"));
    assert!(native.contains("; use_relative_e_distances = 1\n"));
    assert!(native.contains("; wipe_inward = 0\n"));
    assert!(
        native.contains(if matches!(order, NativeOrder::WallsBeforeInfill) {
            "; unsupported_wall_last = 0\n"
        } else {
            "; unsupported_wall_last = 1\n"
        })
    );
    let mut observer = Observer::default();
    for line in native.lines() {
        observer.push(line);
    }
    let strokes = &observer.strokes;
    let first_infill = strokes.iter().position(Stroke::infill).unwrap();
    let last_infill = strokes.iter().rposition(Stroke::infill).unwrap();
    let overhang: Vec<_> = strokes
        .iter()
        .enumerate()
        .filter(|(_, stroke)| stroke.role == "Overhang wall")
        .collect();
    assert!(!overhang.is_empty());
    assert!(overhang.iter().all(|(_, stroke)| stroke.closed()));
    let first_overhang = overhang.first().unwrap().0;
    let last_overhang = overhang.last().unwrap().0;
    match order {
        NativeOrder::InfillBeforeWalls => assert!(last_infill < first_overhang),
        NativeOrder::WallsBeforeInfill => assert!(last_overhang < first_infill),
        NativeOrder::AnchoredBeforeOverhang => {
            let anchored = strokes
                .iter()
                .position(|stroke| stroke.role == "Inner wall" && stroke.closed())
                .unwrap();
            assert!(anchored < first_overhang && last_overhang < first_infill);
        }
    }
    if !matches!(order, NativeOrder::WallsBeforeInfill) {
        assert!(
            overhang
                .windows(2)
                .all(|pair| pair[0].1.footprint() < pair[1].1.footprint())
        );
    }
}

fn check(case: (&str, &[u8], &str), order: NativeOrder) {
    let (label, input, hash) = case;
    native_stream_case::check(
        (
            "arachne-unsupported-wall-last",
            &[
                "PerimeterGenerator.cpp::process_arachne:3093-3111;defer_unsupported_loops:1391-1455",
                "PerimeterGenerator.cpp::traverse_extrusions::inset_idx:563-566",
                "GCode.cpp::deferred perimeter passes:7742-7755,8039-8053,8889-8933",
            ],
        ),
        label,
        input,
        hash,
        (false, |reference, _| check_order(&reference.native, &order)),
    );
}

#[test]
fn unattached_loop_waits_for_infill_complete_output() {
    check(
        (
            "unattached-ledge",
            include_bytes!(
                "../../../../tests/parity/arachne-unsupported-wall-last/unattached-ledge.3mf"
            ),
            "7d8f4a556c3e4f918d56518a29d3ad717f4ee92cc68d226adcaa5d0c555ed270",
        ),
        NativeOrder::InfillBeforeWalls,
    );
}

#[test]
fn disabled_option_preserves_complete_output() {
    check(
        (
            "disabled-control",
            include_bytes!(
                "../../../../tests/parity/arachne-unsupported-wall-last/disabled-control.3mf"
            ),
            "d654a3484bb71e33cb40e6e8729193741944457cfcbdaab6d0b96d0ce88d66d7",
        ),
        NativeOrder::WallsBeforeInfill,
    );
}

#[test]
fn attached_wall_stack_precedes_infill_complete_output() {
    check(
        (
            "attached-stack",
            include_bytes!(
                "../../../../tests/parity/arachne-unsupported-wall-last/attached-stack.3mf"
            ),
            "60d723484161cd2fb56215cb6c2f4d7f2d63cc1fc36544206ac0b0a953f83f49",
        ),
        NativeOrder::AnchoredBeforeOverhang,
    );
}
