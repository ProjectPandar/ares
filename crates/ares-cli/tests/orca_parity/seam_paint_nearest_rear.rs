use std::fs;

use crate::runner::{native_stream_case, repo_root};

const UPSTREAM: &[&str] = &[
    "GCode/SeamPlacer.cpp:850-860,1176-1268,1339-1363,1683-1726",
    "GCode/SeamPlacer.cpp:1774-1818,2027-2080,2161-2204,2345-2423",
];

fn input(mode: &str, printer: &str, paint: &str) -> (String, Vec<u8>, &'static str) {
    let label = format!("{mode}-{printer}-{paint}");
    let file = format!("{label}.3mf");
    let project = fs::read(
        repo_root()
            .join("tests/parity/seam-paint-nearest-rear")
            .join(&file),
    )
    .unwrap();
    let (digest, _) = include_str!("../../../../tests/parity/seam-paint-nearest-rear/SHA256SUMS")
        .lines()
        .filter_map(|line| line.split_once("  "))
        .find(|(_, name)| *name == file)
        .unwrap();
    (label, project, digest)
}

fn compare(mode: &str, printer: &str, paint: &str) {
    let (label, project, digest) = input(mode, printer, paint);
    native_stream_case::check(
        ("seam-paint-nearest-rear", UPSTREAM),
        &label,
        &project,
        digest,
        (false, |_, _| {}),
    );
}

macro_rules! printer_cases {
    ($module:ident, $printer:literal, $mode:literal) => {
        mod $module {
            use super::compare;

            #[test]
            fn plain() {
                compare($mode, $printer, "plain");
            }
            #[test]
            fn front_enforcer() {
                compare($mode, $printer, "front");
            }
            #[test]
            fn rear_blocker() {
                compare($mode, $printer, "rear");
            }
            #[test]
            fn both_annotations() {
                compare($mode, $printer, "both");
            }
            #[test]
            fn side_paint() {
                compare($mode, $printer, "side");
            }
        }
    };
}

printer_cases!(x1c_nearest, "x1c", "nearest");
printer_cases!(k1_nearest, "k1", "nearest");
printer_cases!(mk4_nearest, "mk4", "nearest");
printer_cases!(x1c_rear, "x1c", "back");
printer_cases!(k1_rear, "k1", "back");
printer_cases!(mk4_rear, "mk4", "back");
