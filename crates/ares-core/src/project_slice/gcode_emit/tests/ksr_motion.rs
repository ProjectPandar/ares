#[tokio::test]
async fn ksr_inter_path_travel_retracts_along_wipe_path_and_spiral_lifts() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let lines = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .collect::<Vec<_>>();
    let wipe_start = lines
        .iter()
        .position(|line| *line == "; WIPE_START")
        .unwrap();
    let wipe_end = lines[wipe_start + 1..]
        .iter()
        .position(|line| *line == "; WIPE_END")
        .map(|offset| wipe_start + 1 + offset)
        .unwrap();

    assert_eq!(lines[wipe_start - 2], "G1 X140.645 Y102.949 E.02375");
    assert_eq!(
        &lines[wipe_start + 1..wipe_start + 3],
        [
            "G1 X140.618 Y102.994 E-.02125",
            "G1 X140.353 Y103.632 E-.27626",
        ]
    );
    assert_eq!(lines[wipe_start + 3], "G1 X140.294 Y103.881 E-.1025");
    assert_eq!(lines[wipe_end + 1], "G17");
    assert!(lines[wipe_end + 2].starts_with("G3 Z.6 I"));
    assert!(lines[wipe_end + 2].contains(" J"));
    assert!(lines[wipe_end + 2].ends_with(" P1  F60000"));
    assert_eq!(lines[wipe_end + 3], "G1 X145.539 Y95.848 Z.6");
    assert_eq!(
        lines[wipe_end + 9],
        "G2 X145.766 Y96.281 I3.394 J-1.502 E.01821"
    );
    assert_eq!(
        lines[wipe_end + 23],
        "G2 X155.758 Y90.456 I-6.194 J.091 E.09765"
    );
    let outer_wipe_end = lines
        .iter()
        .enumerate()
        .skip(wipe_end + 1)
        .find(|(_, line)| **line == "; WIPE_END")
        .unwrap()
        .0;
    assert_eq!(lines[outer_wipe_end + 2], "G3 Z.6 I1.188 J-.264 P1  F60000");
    assert_eq!(lines[outer_wipe_end + 3], "G1 X145.539 Y94.166 Z.6");
    let fitted_wipe_start = lines[outer_wipe_end + 1..]
        .iter()
        .position(|line| *line == "; WIPE_START")
        .map(|offset| outer_wipe_end + 1 + offset)
        .unwrap();
    assert_eq!(
        &lines[fitted_wipe_start + 1..fitted_wipe_start + 4],
        [
            "G1 X145.621 Y94.523 E-.1318",
            "G1 X145.756 Y94.862 E-.14599",
            "G1 X145.814 Y95.162 E-.12221",
        ]
    );
    assert_eq!(
        lines[fitted_wipe_start + 6],
        "G3 Z.6 I-.612 J-1.052 P1  F60000"
    );
    assert!(lines.contains(&"G3 X104.96 Y100.092 I.232 J-5.372 E.031"));
    assert!(!lines.contains(&"G1 X136.839 Y100.592 E0"));
    assert!(lines.contains(&"G1 X135.839 Y100.618 E-.4"));
    assert!(!lines.contains(&"G1 X136.839 Y100.592 E-.016"));
    assert!(lines.contains(&"G3 Z.6 I.591 J1.064 P1  F60000"));
    assert!(lines.contains(&"G1 X167.677 Y82.929 Z.6"));
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("G1 X168.396 Y83.649 E"))
    );
    let bottom_surface = lines
        .iter()
        .position(|line| *line == "; FEATURE: Bottom surface")
        .unwrap();
    assert_eq!(lines[bottom_surface + 1], "; LINE_WIDTH: 0.500542");
    assert_eq!(
        &lines[bottom_surface + 91..bottom_surface + 95],
        [
            "G1 X139.407 Y83.135 E.02413",
            "G1 X145.616 Y89.344 E.32744",
            "G1 X145.54 Y89.438 E.00452",
            "G1 X145.092 Y88.869 E.02702",
        ]
    );
    assert_eq!(
        lines
            .iter()
            .find(|line| line.contains("X105.847 Y89.053"))
            .copied(),
        Some("G2 X105.847 Y89.053 I1.094 J1.245 E.01717")
    );
    let bottom_wipe = lines
        .iter()
        .position(|line| *line == "G1 X105.153 Y95.478 E.44148")
        .unwrap();
    assert_eq!(
        &lines[bottom_wipe + 1..bottom_wipe + 7],
        [
            "M204 S6000",
            "G1 E-.11429 F1800",
            "; WIPE_START",
            "G1 F6300",
            "G1 X104.446 Y94.771 E-.28571",
            "; WIPE_END",
        ]
    );
    assert_eq!(lines[bottom_wipe + 8], "G3 Z.6 I.097 J1.213 P1  F60000");
    let first_monotonic_region = lines
        .iter()
        .position(|line| *line == "G1 X99.635 Y137.851 E.15048")
        .unwrap();
    assert_eq!(
        lines[first_monotonic_region + 14],
        "G1 X111.296 Y141.099 E.0564"
    );
}
