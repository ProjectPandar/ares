//! The Type2 final purge: `Print::_make_wipe_tower` skips it when the tower does not reach the
//! last print layer (`Print.cpp:5573-5594`, v2.5.0-beta `345efec64`), and
//! `WipeTowerIntegration::finalize` emits nothing for an empty purge (`GCode.cpp:2141-2151`)
//! (`docs/architecture/wipe-tower2-final-purge.md`). A section oracle: the print tail and the
//! `filament used [mm]` footer line must match Native's; the rest of the output is not compared.

use super::wipe_tower2_rib::{slice_both, slice_fixture};

/// The lines from the last `; stop printing object` to the first `;TYPE:Custom` after it, and
/// the `; filament used [mm]` footer line.
fn print_tail(gcode: &[u8]) -> (Vec<&str>, &str) {
    let lines: Vec<&str> = std::str::from_utf8(gcode).unwrap().lines().collect();
    let start = lines
        .iter()
        .rposition(|line| line.starts_with("; stop printing object"))
        .expect("the G-code labels no object");
    let end = start
        + lines[start..]
            .iter()
            .position(|line| *line == ";TYPE:Custom")
            .expect("no end G-code after the last object");
    let used = lines
        .iter()
        .find(|line| line.starts_with("; filament used [mm]"))
        .expect("no filament used footer");
    (lines[start..end].to_vec(), used)
}

fn assert_tail_matches(label: &str, (native, ares): (Vec<u8>, Vec<u8>)) {
    let (ares_tail, ares_used) = print_tail(&ares);
    let (native_tail, native_used) = print_tail(&native);
    assert_eq!(ares_tail, native_tail, "{label}: print tail");
    assert_eq!(ares_used, native_used, "{label}: filament used");
}

#[test]
fn tower_below_the_last_layer_skips_the_final_purge_like_native() {
    assert_tail_matches(
        "final-purge-short-tower",
        slice_fixture(
            "tests/parity/mk4-layer-change/mk4base.3mf",
            "final-purge-short-tower",
            &[],
        ),
    );
}

#[test]
fn tower_up_to_the_last_layer_prints_the_final_purge_like_native() {
    assert_tail_matches(
        "final-purge-full-tower",
        slice_both("final-purge-full-tower", &[]),
    );
}
