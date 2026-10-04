//! `AdaptivePAProcessor::process_layer` feedrate lookahead boundaries.

pub(super) fn feedrate(line: &str) -> Option<f64> {
    let value = line.strip_prefix("G1 F")?;
    super::number::parse(value).map(|value| value / 60.0)
}

pub(super) struct Speeds {
    pub(super) current: f64,
    pub(super) next: f64,
    pub(super) maximum: f64,
}

pub(super) fn lookahead(lines: &[&str], current: f64) -> Speeds {
    let mut speeds = Speeds {
        current,
        next: 0.0,
        maximum: 0.0,
    };
    let mut extruded = false;
    for (index, line) in lines.iter().enumerate() {
        let xy_move = line.starts_with("G1 ") && line.contains('X') && line.contains('Y');
        if !extruded && xy_move && line.contains('E') {
            extruded = true;
            continue;
        }
        if (extruded && xy_move && !line.contains('E')) || line.contains("WIPE") {
            break;
        }
        if line.starts_with("; PA_CHANGE") && line.contains("RC:1") {
            break;
        }
        if let Some(feedrate) = feedrate(line) {
            if index == 0 {
                speeds.current = feedrate;
            }
            speeds.maximum = speeds.maximum.max(feedrate);
            if speeds.next < 1e-4 {
                speeds.next = feedrate;
            }
        }
    }
    if speeds.maximum <= 0.0 {
        speeds.maximum = speeds.current;
    }
    speeds
}
