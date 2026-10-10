use std::collections::BTreeMap;

#[derive(Debug)]
pub(super) struct Segment {
    pub(super) key: String,
    pub(super) role: String,
    pub(super) length: f64,
    pub(super) e: f64,
}

pub(super) fn observe(bytes: &[u8]) -> Vec<Segment> {
    let mut role = "";
    let mut layer = 0;
    let mut relative = false;
    let mut position = [0.0_f64; 2];
    let mut absolute_e = 0.0;
    let mut segments = Vec::new();
    for line in std::str::from_utf8(bytes).unwrap().lines() {
        if let Some(value) = line.strip_prefix(";TYPE:") {
            role = value;
        }
        if line == ";LAYER_CHANGE" {
            layer += 1;
        }
        let mut words = line.split(';').next().unwrap().split_ascii_whitespace();
        let command = words.next();
        if command == Some("M83") {
            relative = true;
        } else if command == Some("M82") {
            relative = false;
        }
        if !matches!(command, Some("G0" | "G1" | "G92")) {
            continue;
        }
        let mut next = position;
        let mut xy = false;
        let mut extrusion = None;
        for word in words {
            if let Some(value) = word.strip_prefix('X') {
                next[0] = value.parse().unwrap();
                xy = true;
            }
            if let Some(value) = word.strip_prefix('Y') {
                next[1] = value.parse().unwrap();
                xy = true;
            }
            if let Some(value) = word.strip_prefix('E') {
                extrusion = Some(value.parse::<f64>().unwrap());
            }
        }
        let e = extrusion.map(|e| {
            let delta = if relative { e } else { e - absolute_e };
            absolute_e = e;
            delta
        });
        if command != Some("G92") && xy && e.is_some_and(|e| e > 0.0) {
            segments.push(Segment {
                key: format!("{layer}/{role}/{:?}/{:?}", position, next),
                role: role.to_owned(),
                length: (next[0] - position[0]).hypot(next[1] - position[1]),
                e: e.unwrap(),
            });
        }
        position = next;
    }
    assert!(
        !segments.is_empty(),
        "missing actual positive-E XY extrusion"
    );
    segments
}

pub(super) fn role_counts(segments: &[Segment]) -> BTreeMap<&str, usize> {
    let mut counts = BTreeMap::new();
    for segment in segments {
        *counts.entry(segment.role.as_str()).or_default() += 1;
    }
    counts
}

fn eligible(role: &str) -> bool {
    matches!(
        role,
        "Internal solid infill" | "Top surface" | "Bottom surface"
    )
}

pub(super) fn verify_pair(off: &[Segment], on: &[Segment], active: bool) -> usize {
    let mut matching = BTreeMap::<&str, Vec<&Segment>>::new();
    for segment in off {
        matching.entry(&segment.key).or_default().push(segment);
    }
    let mut reduced = 0;
    let mut unchanged_wall = 0;
    for segment in on {
        let Some(candidates) = matching.get_mut(segment.key.as_str()) else {
            continue;
        };
        let Some(baseline) = candidates.pop() else {
            continue;
        };
        if active && eligible(&segment.role) && segment.length > 0.0 && segment.length < 20.0 {
            reduced += usize::from(segment.e < baseline.e);
        } else {
            assert_eq!(
                segment.e, baseline.e,
                "unexpected correction: {}",
                segment.key
            );
        }
        unchanged_wall += usize::from(matches!(segment.role.as_str(), "Inner wall" | "Outer wall"));
    }
    assert!(
        unchanged_wall > 0,
        "missing matched unmodified wall extrusion"
    );
    if active {
        assert!(
            reduced > 0,
            "no matched interior eligible segment has actual E reduction"
        );
    } else {
        assert_eq!(
            off.len(),
            on.len(),
            "disabled controls changed extrusion segment count"
        );
        for (baseline, segment) in off.iter().zip(on) {
            assert_eq!(
                baseline.key, segment.key,
                "disabled controls changed path order"
            );
            assert_eq!(baseline.e, segment.e, "disabled controls changed emitted E");
        }
    }
    reduced
}

pub(super) fn require_roles(segments: &[Segment]) {
    let counts = role_counts(segments);
    for role in [
        "Internal solid infill",
        "Top surface",
        "Bottom surface",
        "Outer wall",
    ] {
        assert!(
            counts.contains_key(role),
            "required actual {role} extrusion absent: {counts:?}"
        );
    }
}
