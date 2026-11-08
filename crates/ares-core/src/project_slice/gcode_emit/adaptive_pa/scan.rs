//! `AdaptivePAProcessor::process_layer` feedrate lookahead boundaries.

pub(super) fn feedrate(line: &str) -> Option<f64> {
    let value = line.strip_prefix("G1 F")?;
    super::number::parse(value).map(|value| value / 60.0)
}

pub(super) fn move_feedrate(line: &str) -> Option<f64> {
    if !["G0 ", "G1 ", "G2 ", "G3 "]
        .iter()
        .any(|prefix| line.starts_with(prefix))
    {
        return None;
    }
    let position = line.find(" F")?;
    if line.find(';').is_some_and(|comment| comment < position) {
        return None;
    }
    let value = &line[position + 2..];
    if !value.as_bytes().first()?.is_ascii_digit() {
        return None;
    }
    super::number::parse(value)
        .map(|value| value / 60.0)
        .filter(|value| *value > 0.0)
}

pub(super) struct Speeds {
    pub(super) current: f64,
    pub(super) next: f64,
    pub(super) maximum: f64,
}

pub(super) fn lookahead(lines: &[&str], current: f64, mut modal: f64) -> Speeds {
    let mut speeds = Speeds {
        current,
        next: 0.0,
        maximum: 0.0,
    };
    let mut extruded = false;
    for (index, line) in lines.iter().enumerate() {
        if !extruded && let Some(feedrate) = move_feedrate(line) {
            modal = feedrate;
        }
        let xy_move = line.starts_with("G1 ") && line.contains('X') && line.contains('Y');
        if !extruded && xy_move && line.contains('E') {
            extruded = true;
            if speeds.maximum == 0.0 && modal > 0.0 {
                speeds.maximum = modal;
                speeds.next = modal;
                speeds.current = modal;
            }
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
