//! Test legs for the support-ironing pattern domain (split mechanically
//! from the parent module; helpers live there).

use super::{leg, leg_with_project, sparse_project};

/// Control: rectilinear is the implemented default path and must stay
/// byte-identical to Native before and after the domain slice.
#[test]
fn support_ironing_pattern_rectilinear_control() {
    leg("rect-control", "rectilinear");
}

/// Control: the legacy "zig-zag" alias loads through
/// `PrintConfig.cpp:8206` normalization on the Native side and through the
/// serde alias on the ares side; both must keep matching byte-for-byte.
#[test]
fn support_ironing_pattern_zig_zag_control() {
    leg("zig-zag-control", "zig-zag");
}

/// Domain: grid ironing paths (`FillBase.cpp` ipGrid) must match Native.
#[test]
fn support_ironing_pattern_grid() {
    leg("grid", "grid");
}

/// Domain: monotonic ironing paths (`FillBase.cpp` ipMonotonic).
#[test]
fn support_ironing_pattern_monotonic() {
    leg("monotonic", "monotonic");
}

/// Domain: honeycomb ironing paths (`FillBase.cpp` ipHoneycomb).
#[test]
fn support_ironing_pattern_honeycomb() {
    leg("honeycomb", "honeycomb");
}

/// Domain: triangles ironing paths (`FillBase.cpp` ipTriangles).
#[test]
fn support_ironing_pattern_triangles() {
    leg("triangles", "triangles");
}

/// Domain: gyroid ironing paths (`FillBase.cpp` ipGyroid).
#[test]
fn support_ironing_pattern_gyroid() {
    leg("gyroid", "gyroid");
}

/// Domain: cubic ironing paths (`FillBase.cpp` ipCubic).
#[test]
fn support_ironing_pattern_cubic() {
    leg("cubic", "cubic");
}

/// Domain: quarter-cubic ironing paths (`FillBase.cpp` ipQuarterCubic).
#[test]
fn support_ironing_pattern_quartercubic() {
    leg("quartercubic", "quartercubic");
}

/// Domain: 3D honeycomb ironing paths (`FillBase.cpp` ip3DHoneycomb).
#[test]
fn support_ironing_pattern_three_d_honeycomb() {
    leg("3dhoneycomb", "3dhoneycomb");
}

/// Domain: tri-hexagon ironing paths (`PrintConfig.cpp:238` maps to ipStars).
#[test]
fn support_ironing_pattern_tri_hexagon() {
    leg("tri-hexagon", "tri-hexagon");
}

/// Domain: TPMS-D ironing paths (`FillBase.cpp` ipTpmsD).
#[test]
fn support_ironing_pattern_tpmsd() {
    leg("tpmsd", "tpmsd");
}

/// Domain: TPMS-FK ironing paths (`FillBase.cpp` ipTpmsFK).
#[test]
fn support_ironing_pattern_tpmsfk() {
    leg("tpmsfk", "tpmsfk");
}

/// Domain: lateral honeycomb ironing paths (`FillBase.cpp` ipLateralHoneycomb).
#[test]
fn support_ironing_pattern_lateral_honeycomb() {
    leg("lateral-honeycomb", "lateral-honeycomb");
}

/// Domain: aligned rectilinear ironing paths (`FillBase.cpp`
/// ipAlignedRectilinear shares `fill_surface_by_lines`).
#[test]
fn support_ironing_pattern_aligned_rectilinear() {
    leg("alignedrectilinear", "alignedrectilinear");
}

/// Domain: monotonic line ironing paths (`FillBase.cpp` ipMonotonicLine).
#[test]
fn support_ironing_pattern_monotonic_line() {
    leg("monotonicline", "monotonicline");
}

/// Control: concentric is the pre-slice implemented arm and must stay
/// byte-identical to Native before and after the domain slice.
#[test]
fn support_ironing_pattern_concentric_control() {
    leg("concentric-control", "concentric");
}

/// Domain: lateral lattice ironing paths — `FillLateralLattice` reached
/// from the 3MF load; pins the full-arc degeneracy handling
/// (`FillBase.cpp:474` equal-index walks the whole contour instead of
/// asserting).
#[test]
fn support_ironing_pattern_lateral_lattice() {
    leg("lateral-lattice", "lateral-lattice");
}

/// Control: sparse lateral-lattice (`FillLateralLattice` sparse path
/// through the same `fill/connect` stage) must stay byte-identical to
/// Native while the contour assert is removed.
#[test]
fn sparse_lateral_lattice_control() {
    leg_with_project(
        "sparse-lattice-control",
        "lateral-lattice",
        sparse_project("lateral-lattice"),
    );
}

/// Domain: consistent zig-zag ironing — `FillZigZag` under the canonical
/// enum key (`"zigzag"`, `PrintConfig.cpp:232`; the hyphenated legacy
/// `"zig-zag"` migrates to rectilinear at `PrintConfig.cpp:8206`). The
/// vertical-line grid anchors on the extended dummy support box
/// (`SupportCommon.cpp:1433`/`:1880`); Native emits identical ironing
/// toolpaths for all three consistent patterns, differing only in the
/// config echo (127 E-lines each, probe 2026-10-14).
#[test]
fn support_ironing_pattern_zig_zag() {
    leg("zig-zag", "zigzag");
}

/// Domain: consistent cross-zag ironing — `FillCrossZag`; at the ironing
/// call site every `FillParams` phase/symmetric control keeps its default,
/// matching the zig-zag leg byte-for-byte in Native.
#[test]
fn support_ironing_pattern_cross_zag() {
    leg("cross-zag", "crosszag");
}

/// Domain: consistent locked-zag ironing — `FillLockedZag`; the lock-region
/// extrusion path is unreachable here (only `fill_surface_extrusion` is
/// overridden, the Polylines overload serves the base rectilinear
/// traversal), so this pins the same consistent traversal as zig-zag.
#[test]
fn support_ironing_pattern_locked_zag() {
    leg("locked-zag", "lockedzag");
}

/// Domain: unknown enum keys silently fall to the option default
/// (`Config.cpp:654-672` substitution under the `EnableSilent` 3MF load
/// rule, `Model.cpp:259`) — even the gcode config echo carries the
/// substituted default.
#[test]
fn support_ironing_pattern_unknown_key_cross_zag() {
    leg("unknown-cross-zag", "cross-zag");
}

/// Domain: same substitution for an arbitrary unknown key — Native slices
/// with the default pattern and no warning (probe 2026-10-14:
/// 719 E-lines, echo `rectilinear`).
#[test]
fn support_ironing_pattern_unknown_key_bogus() {
    leg("unknown-bogus", "bogus-pattern");
}

/// Domain: `FillLine` ironing — vertical scan family with endpoint
/// extension and the oscillation-based connect (`FillLine.cpp:13-110`).
#[test]
fn support_ironing_pattern_line() {
    leg("line", "line");
}

/// Domain: `FillCrossHatch` ironing — the z-phase layered pattern
/// (`FillCrossHatch.cpp:14-233`), keyed on the support layer print_z.
#[test]
fn support_ironing_pattern_crosshatch() {
    leg("crosshatch", "crosshatch");
}

/// Domain: `OctagramSpiral` ironing — plane-path filler, centered
/// pattern (`FillPlanePath.cpp:62-171`, density-1 snug-bounds path).
#[test]
fn support_ironing_pattern_octagramspiral() {
    leg("octagramspiral", "octagramspiral");
}

/// Domain: `ArchimedeanChords` ironing — plane-path filler, center
/// shift (`FillPlanePath.hpp:65`, `centered() == true`; density-1
/// snug-bounds path).
#[test]
fn support_ironing_pattern_archimedeanchords() {
    leg("archimedeanchords", "archimedeanchords");
}

/// Domain: `HilbertCurve` ironing — plane-path filler, min-corner
/// shift (`FillPlanePath.hpp:72`, `centered() == false`; density-1
/// snug-bounds path).
#[test]
fn support_ironing_pattern_hilbertcurve() {
    leg("hilbertcurve", "hilbertcurve");
}
