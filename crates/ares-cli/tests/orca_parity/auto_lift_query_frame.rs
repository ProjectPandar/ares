//! GCode.cpp:7604 queries Auto-lift overhangs in current-plate coordinates.
//! Object labels: GCode.cpp:7606-7609,11356-11371 at
//! `345efec64c7b95a05ce5cf96df0dde66a158835c`.
//! Native activity: GCodeWriter.cpp:1262-1272; GCode/GCodeProcessor.cpp:2823.

use crate::runner::native_stream_case;

const DIR: &str = "auto-lift-query-frame";
const LABEL: &str = "p1s-bylayer-two-objects";
const SHA: &str = "f22c2132032860447369a3b8d29291230108f20ef1030cb532447cfb90ae31dd";
const PROJECT: &[u8] = include_bytes!("../../../../tests/parity/auto-lift-query-frame/project.3mf");
const UPSTREAM: &[&str] = &[
    "GCode.cpp::needs_retraction:7593-7604;Auto Spiral/Slope:7607-7615,7635-7647",
    "GCodeWriter.hpp::get_xy_offset:98-99",
];

fn assert_native_witness(native: &str) {
    for marker in [
        "; printer_model = Bambu Lab P1S\n",
        "; print_sequence = by layer\n",
        "; z_hop_types = Auto Lift\n",
        "; model label id: 45,56\n",
        "; object ids of layer 1 start: 45,56\n",
    ] {
        assert!(native.contains(marker), "Native witness missing {marker:?}");
    }
    let mut objects = std::collections::BTreeMap::new();
    for label in native
        .lines()
        .filter_map(|line| line.strip_prefix("; printing object "))
    {
        let (object, identity) = label.rsplit_once(" id:").unwrap();
        if let Some(previous) = objects.insert(object, identity) {
            assert_eq!(identity, previous, "Native object/copy label is stable");
        }
    }
    assert!(objects.len() > 1, "Native multiple-object input");
    let identities: std::collections::BTreeSet<_> = objects.values().collect();
    assert_eq!(
        identities.len(),
        objects.len(),
        "distinct Native object/copy labels"
    );
    assert!(
        native
            .lines()
            .zip(native.lines().skip(1))
            .any(|(plane, line)| {
                let arc = line.split_ascii_whitespace();
                plane.split(';').next().unwrap().trim() == "G17"
                    && arc.clone().next() == Some("G3")
                    && ['Z', 'I', 'J', 'P', 'F']
                        .into_iter()
                        .all(|axis| arc.clone().any(|word| word.starts_with(axis)))
                    && !arc.clone().any(|word| word.starts_with('E'))
            }),
        "Native spiral lift activity"
    );
    assert!(
        native.lines().any(|line| {
            let words = line.split_ascii_whitespace();
            words.clone().next() == Some("M73")
                && ['P', 'R']
                    .into_iter()
                    .all(|axis| words.clone().any(|word| word.starts_with(axis)))
        }),
        "Native normal progress activity"
    );
}

#[test]
fn auto_lift_query_frame_p1s() {
    native_stream_case::check(
        (DIR, UPSTREAM),
        LABEL,
        PROJECT,
        SHA,
        (false, |reference, replay| {
            assert_native_witness(&reference.native);
            assert_native_witness(&replay.native);
        }),
    );
}
