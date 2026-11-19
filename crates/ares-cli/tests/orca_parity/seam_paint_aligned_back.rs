use std::fs;

use crate::runner::{native_stream_case, repo_root};

const UPSTREAM: &[&str] = &[
    "GCode/SeamPlacer.cpp:196-199,850-860,957-978,1176-1268",
    "GCode/SeamPlacer.cpp:1339-1350,1498-1627,1802-1818,1883-2081,2150-2204",
];

fn compare(printer: &str, variant: &str) {
    let label = format!("{printer}-{variant}");
    let file = format!("{label}.3mf");
    let project = fs::read(
        repo_root()
            .join("tests/parity/seam-paint-aligned-back")
            .join(&file),
    )
    .unwrap();
    let (digest, _) = include_str!("../../../../tests/parity/seam-paint-aligned-back/SHA256SUMS")
        .lines()
        .filter_map(|line| line.split_once("  "))
        .find(|(_, name)| *name == file)
        .unwrap();
    native_stream_case::check(
        ("seam-paint-aligned-back", UPSTREAM),
        &label,
        &project,
        digest,
        (false, |_, _| {}),
    );
}

macro_rules! printer_cases {
    ($printer:ident, $label:literal) => {
        mod $printer {
            use super::compare;

            #[test]
            fn unpainted() {
                compare($label, "plain");
            }
            #[test]
            fn front_enforcer() {
                compare($label, "front");
            }
            #[test]
            fn rear_blocker() {
                compare($label, "rear");
            }
            #[test]
            fn both_annotations() {
                compare($label, "both");
            }
            #[test]
            fn side_paint() {
                compare($label, "side");
            }
        }
    };
}

printer_cases!(x1c, "x1c");
printer_cases!(k1, "k1");
printer_cases!(mk4, "mk4");
