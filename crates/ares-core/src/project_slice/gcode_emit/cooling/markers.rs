//! Pressure/role marker stripping — the final output pass removing the
//! `;_EXTRUDE_*`, `;_EXTERNAL_PERIMETER`, `;_WIPE`, and `;_EXTRUSION_ROLE`
//! tags plus the modal-feedrate dedup.

/// Remove the `;_EXTRUDE_SET_SPEED` / `;_EXTRUDE_END` /
/// `;_EXTERNAL_PERIMETER` / `;_EXTRUSION_ROLE` / `;_WIPE` scaffolding —
/// upstream's processor consumes the tags after the PressureEqualizer
/// (`GCodeProcessor` treats them as internal markers; the exported file
/// carries none).
pub(super) fn strip_pressure_markers(gcode: &str) -> String {
    const TAGS: [&str; 5] = [
        ";_EXTRUDE_SET_SPEED",
        ";_EXTERNAL_PERIMETER",
        ";_EXTRUDE_END",
        ";_EXTRUSION_ROLE",
        ";_WIPE",
    ];
    let mut out = String::with_capacity(gcode.len());
    // The processor-side marker consumption also drops F-only lines that do
    // not change the modal feedrate (the equalizer re-emits `G1 F<n>` per
    // modified line; when n equals the current feedrate the line vanishes
    // from the exported stream — verified against the 2bDWHY skirt where
    // upstream emits one F3000 for the whole block).
    let mut current_feedrate: Option<i64> = None;
    for line in gcode.lines() {
        if line == ";_EXTRUDE_END" {
            continue;
        }
        let mut cleaned = line.to_owned();
        let mut changed = false;
        for tag in TAGS {
            if cleaned.contains(tag) {
                cleaned = cleaned.replace(tag, "");
                changed = true;
            }
        }
        if changed {
            cleaned = cleaned.trim_end().to_owned();
            if cleaned.is_empty() {
                continue;
            }
        }
        // The processor-side modal feedrate tracks EVERY F word (the
        // GCodeReader updates `m_feedrate` on any G0/G1 carrying F —
        // travels included), so a travel's `F15000` between two `G1
        // F1200` lines keeps the second one: it changes the modal rate
        // back from 15000.
        if let Some(feedrate) = line_feedrate_word(&cleaned) {
            if bare_feedrate_word(&cleaned).is_some() && current_feedrate == Some(feedrate) {
                continue;
            }
            current_feedrate = Some(feedrate);
        }
        out.push_str(&cleaned);
        out.push('\n');
    }
    out
}

/// A `G1 F<n>` line with no other words (the equalizer's re-emitted
/// speed-setting line after tag stripping).
pub(super) fn bare_feedrate_word(line: &str) -> Option<i64> {
    let rest = line.strip_prefix("G1 F")?;
    if rest.is_empty() || !rest.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    rest.parse().ok()
}

/// The F word of any G0/G1 line (the processor's modal feedrate source —
/// the F may sit after other axis words, e.g. `G1 X10 Y10 F1500`).
fn line_feedrate_word(line: &str) -> Option<i64> {
    let rest = line
        .strip_prefix("G0")
        .or_else(|| line.strip_prefix("G1"))?;
    for word in rest.split_whitespace() {
        if let Some(digits) = word.strip_prefix('F')
            && !digits.is_empty()
            && digits.bytes().all(|b| b.is_ascii_digit())
        {
            return digits.parse().ok();
        }
    }
    None
}
