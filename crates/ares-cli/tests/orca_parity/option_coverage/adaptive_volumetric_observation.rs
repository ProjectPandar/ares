use std::collections::BTreeMap;

#[derive(Debug)]
pub(super) struct Segment {
    pub(super) key: String,
    pub(super) role: String,
    pub(super) layer: usize,
    pub(super) feed: f64,
    pub(super) e: f64,
}

pub(super) fn observe(bytes: &[u8]) -> Vec<Segment> {
    let mut role = "";
    let mut layer = 0;
    let mut relative = false;
    let mut position = [0.0_f64; 2];
    let mut feed = 0.0;
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
            if let Some(value) = word.strip_prefix('F') {
                feed = value.parse().unwrap();
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
                layer,
                feed,
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

pub(super) fn verify_pair(off: &[Segment], on: &[Segment], active: bool) -> usize {
    assert_eq!(off.len(), on.len(), "fit changed extrusion count");
    let mut reduced = 0;
    for (baseline, segment) in off.iter().zip(on) {
        assert_eq!(baseline.key, segment.key, "fit changed geometry/path order");
        assert_eq!(baseline.e, segment.e, "fit changed E");
        if active {
            assert!(segment.feed <= baseline.feed, "fit raised feed");
            reduced += usize::from(segment.layer > 1 && segment.feed < baseline.feed);
        } else {
            assert_eq!(baseline.feed, segment.feed, "inactive fit changed feed");
        }
    }
    if active {
        assert!(
            reduced > 0,
            "Native/ARES active fit has no nonfirst-layer F witness"
        );
    }
    reduced
}
