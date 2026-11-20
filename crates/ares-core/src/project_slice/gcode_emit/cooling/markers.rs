//! Pressure/role marker stripping — the final output pass removing the
//! `;_EXTRUDE_*`, `;_EXTERNAL_PERIMETER`, `;_WIPE`, and `;_EXTRUSION_ROLE`
//! tags. Feedrate emission is decided by the cooling rewrite.

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
        out.push_str(&cleaned);
        out.push('\n');
    }
    out
}
