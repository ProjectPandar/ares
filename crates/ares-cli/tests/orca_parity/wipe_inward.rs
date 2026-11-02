//! Public Native premises for beta `WipePathHelpers` and `GCode.cpp:8729-8801`.

use crate::runner::native_stream_case;

#[derive(Clone)]
struct Wipe {
    role: String,
    wall: Vec<(f64, f64)>,
    moves: Vec<(f64, f64)>,
    inner_before: bool,
    nonzero_e: bool,
    negative_e: bool,
}

impl Wipe {
    fn closed_outer(&self) -> bool {
        self.role == "Outer wall"
            && self.wall.len() >= 4
            && self.wall.first() == self.wall.last()
            && !self.moves.is_empty()
    }

    fn interior_move(&self) -> bool {
        let (min, max) = self.bounds();
        self.moves
            .iter()
            .any(|&(x, y)| x > min.0 && x < max.0 && y > min.1 && y < max.1)
    }

    fn ordinary_boundary_path(&self) -> bool {
        let (min, max) = self.bounds();
        self.moves.iter().all(|&(x, y)| {
            ((x == min.0 || x == max.0) && y >= min.1 && y <= max.1)
                || ((y == min.1 || y == max.1) && x >= min.0 && x <= max.0)
        })
    }

    fn bounds(&self) -> ((f64, f64), (f64, f64)) {
        self.wall.iter().fold(
            (
                (f64::INFINITY, f64::INFINITY),
                (f64::NEG_INFINITY, f64::NEG_INFINITY),
            ),
            |(min, max), &(x, y)| ((min.0.min(x), min.1.min(y)), (max.0.max(x), max.1.max(y))),
        )
    }
}

fn word(line: &str, prefix: char) -> Option<f64> {
    line.split_whitespace()
        .find_map(|word| word.strip_prefix(prefix))
        .map(|value| value.parse().unwrap())
}

#[derive(Default)]
struct Observer<'a> {
    result: Vec<Wipe>,
    active: Option<Wipe>,
    position: (f64, f64),
    role: &'a str,
    wall_role: &'a str,
    wall: Vec<(f64, f64)>,
    inner_seen: bool,
    inner_before: bool,
    contiguous: bool,
}

impl<'a> Observer<'a> {
    fn push(&mut self, line: &'a str) {
        if let Some(value) = line.strip_prefix(";TYPE:") {
            self.role = value;
        }
        if line == ";LAYER_CHANGE" {
            self.inner_seen = false;
        }
        match line {
            ";WIPE_START" => {
                self.active = Some(Wipe {
                    role: self.wall_role.to_owned(),
                    wall: self.wall.clone(),
                    moves: Vec::new(),
                    inner_before: self.inner_before,
                    nonzero_e: false,
                    negative_e: false,
                })
            }
            ";WIPE_END" => {
                self.result.push(self.active.take().unwrap());
                self.contiguous = false;
            }
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
        let e = word(code, 'E').unwrap_or(0.0);
        self.position = (x.unwrap_or(self.position.0), y.unwrap_or(self.position.1));
        let moved = x.is_some() || y.is_some();
        if let Some(wipe) = &mut self.active {
            wipe.nonzero_e |= e != 0.0;
            wipe.negative_e |= e < 0.0;
            if moved {
                wipe.moves.push(self.position);
            }
        } else if moved && e > 0.0 {
            if !self.contiguous || self.wall_role != self.role {
                self.wall = vec![before];
                self.inner_before = self.inner_seen;
            }
            self.inner_seen |= self.role == "Inner wall";
            self.wall.push(self.position);
            self.wall_role = self.role;
            self.contiguous = true;
        } else if moved {
            self.contiguous = false;
        }
    }
}

fn observe(native: &str) -> Vec<Wipe> {
    let mut observer = Observer::default();
    for line in native.lines() {
        observer.push(line);
    }
    observer.result
}

enum Premise {
    AcceptedOffset,
    Disabled,
    NoEarlierAnchor,
}

fn check(case: (&str, &[u8], &str), premise: Premise) {
    let (label, bytes, hash) = case;
    native_stream_case::check(
        (
            "wipe-inward",
            &[
                "PrintConfig.cpp:6670-6698;WipePathHelpers.cpp:21-884",
                "GCode.cpp:668-756,8494,8729-8801,8847-8854,8903-8929",
            ],
        ),
        label,
        bytes,
        hash,
        (false, |reference, _| {
            let native = &reference.native;
            assert!(native.contains("; wall_generator = classic\n"));
            assert!(native.contains("; use_relative_e_distances = 1\n"));
            assert!(native.contains("; wipe_on_loops = 0\n"));
            assert!(native.contains(if matches!(premise, Premise::Disabled) {
                "; wipe_inward = 0\n"
            } else {
                "; wipe_inward = 1\n"
            }));
            let wipes = observe(native);
            let accepted = |wipe: &Wipe| {
                wipe.closed_outer() && wipe.inner_before && !wipe.nonzero_e && wipe.interior_move()
            };
            match premise {
                Premise::AcceptedOffset => assert!(wipes.iter().any(accepted)),
                Premise::Disabled | Premise::NoEarlierAnchor => {
                    assert!(!wipes.iter().any(accepted));
                    assert!(wipes.iter().any(|wipe| wipe.closed_outer()
                        && wipe.negative_e
                        && wipe.ordinary_boundary_path()));
                    if matches!(premise, Premise::NoEarlierAnchor) {
                        assert!(!native.contains(";TYPE:Inner wall\n"));
                        assert!(
                            !wipes
                                .iter()
                                .any(|wipe| wipe.closed_outer() && wipe.interior_move())
                        );
                    }
                }
            }
        }),
    );
}

#[test]
fn enabled_inner_outer_offset_matches_native_complete_output() {
    check(
        (
            "enabled-inner-outer",
            include_bytes!("../../../../tests/parity/wipe-inward/enabled-inner-outer.3mf"),
            "e160622e1c76a54796dee510a8970e65d28c145960b20315f0a32ec88db432bf",
        ),
        Premise::AcceptedOffset,
    );
}

#[test]
fn disabled_wipe_inward_matches_native_complete_output() {
    check(
        (
            "disabled-control",
            include_bytes!("../../../../tests/parity/wipe-inward/disabled-control.3mf"),
            "2c904340328ca20f97e29d046a0b211730db12f1683238ca093842da49b95ed1",
        ),
        Premise::Disabled,
    );
}

#[test]
fn single_wall_keeps_ordinary_stored_wipe_path() {
    check(
        (
            "single-wall-no-anchor",
            include_bytes!("../../../../tests/parity/wipe-inward/single-wall-no-anchor.3mf"),
            "b11d24c6414ddefd57e6a31452dd1d5e197f3864cdb5ad57e18a017c2a1b2a12",
        ),
        Premise::NoEarlierAnchor,
    );
}
