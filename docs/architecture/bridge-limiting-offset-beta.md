# Beta bridge limiting-boundary offset units

## Source boundary

Rewrite OrcaSlicer beta `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`src/libslic3r/PrintObject.cpp:3895–3903`, inside
`PrintObject::bridge_over_infill()` (`:3088–4274`). The source builds boundary
polylines from total fill expanded by `1.3f * flow.scaled_spacing()` (`:3898`)
and limiting area expanded by `0.3f * flow.scaled_spacing()` (`:3902`).
`Flow.hpp:71` defines the scaled spacing as an integer coordinate distance.

The Rust destination is
`ares-core/src/project_slice/prepare_infill/bridge_over_infill/candidate_boundary_polylines.rs`
(`prepare_candidate_boundary_polylines_using`, originally lines 47–54) and its
caller in `transaction/candidate_expansion/layers.rs:261–267`. The existing
module is a compatibility shell for that upstream boundary, not an independent
Ares bridge design.

## Decision

Both expansion operands must use scaled coordinate distances. The existing
limiting operand multiplies the unscaled millimetre `Flow.spacing` by `0.3`,
while the total-fill operand already uses `scaled_spacing`. Remove the redundant
unscaled-spacing argument and use the source's float multiplication for both
coefficients. Do not alter surrounding bridge algorithms.

The representative frozen project is
`tests/parity/seam-gap-fill-group-key/absolute-equal.3mf`, SHA256
`7620eafdbeb793c46cfcf7745e3e8c94d5bdbd76cd8f7e8aa07f5e2ccd6dde8b`.
Read-only observation found equal complete region geometry before
`bridge_over_infill`, but two Native holes versus one Rust hole in region 0 on
layer 67 afterward. Rust's two layer-68 candidates execute the limiting-boundary
path with spacing approximately `0.439871788` mm and scaled spacing `439871`:
the current operand is approximately `0.1319615` coordinate units instead of
`131961.3`. This establishes the executed unit mismatch, not complete-output
causality or a guarantee that every inherited output difference disappears.

## Included and deferred behavior

Included: the two source expansion coefficients and their coordinate units;
public byte-in/byte-out complete-output parity for the representative project;
bridge/infill-family and same-inventory baseline/candidate comparisons.

Deferred: ExPolygon component filtering versus individual ring filtering
(`PrintObject.cpp:3876–3887`), turning-pattern sampling/contact restoration,
bridge direction selection, general geometry cleanup, and other owners' writer,
M73, arc-mode, or configuration behavior. None is silently repaired here.
The eleven downstream Rectilinear symptoms are not assumed to share one cause.

## Verification gate

Commit this decision before a public-API complete-output RED, then commit the
minimal source port separately. Use the standard parity harness and its ordered
complete-output comparator; do not filter roles, normalize feeds/M73, or accept
unordered geometry. Retain raw outputs and repeatable artifacts externally.
Standard test Native runs are in scope; additional Native debugger/observer
runs are not.

The representative must genuinely pass before calling this GREEN. Any inherited
full-output mismatch remains a failure and must be reported, not weakened.
Before landing, require bridge/infill-family and the `9f6ee320d` baseline
comparison with zero PASS-to-FAIL transitions, plus rustfmt and all-target Clippy
with warnings denied. A failure blocks completion rather than justifying a
second speculative repair.
