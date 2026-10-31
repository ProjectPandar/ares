# SPEC — Arachne infill-boundary resolution prerequisite

Status: **SPEC APPROVED; implementation strategy adjusted by the operator below.**
Base main for the mergeable prerequisite: `96328f269`.
`worker4/arachne-infill-resolution` retains only the extracted None public RED,
with its original assertions/helper unchanged except the isolated matrix path;
`048cdb619` precedes approved production cherry-pick `c33e781dd`.
The eight non-None REDs stay registered only on the original counterbore branch.
Upstream pin: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
This supersedes the proposed ArcFitter/Circle production prerequisite, whose
same-input gate failed. It ports one source-owned PerimeterGenerator argument
handoff, not a new fitter, fill connector or Ares pipeline feature.
[Paired input STOP](arc-counterbore-paired-input-diagnosis.md) and
[resolution ARD](ard-arachne-infill-boundary-resolution.md).

## Source-cited owning boundary

Paths are relative to `OrcaSlicer/src/libslic3r/` at the pin:

| Source / lines | Responsibility |
| --- | --- |
| `PerimeterGenerator.hpp:125–138`, constructor, specifically `:135` | `m_scaled_resolution = scaled<double>(resolution > EPSILON ? resolution : EPSILON)`; keep the 1e-4 floor and active coordinate scale. |
| `PerimeterGenerator.cpp::process_arachne:2125` | Local `surface_simplify_resolution` is 0.2 times the member resolution only with arc-fitting enabled and no fuzzy-skin contour. |
| Same function `:2148–2150` | That local value belongs to **wall-input** `surface.expolygon.simplify_p(surface_simplify_resolution)` before WallToolPaths. |
| `Arachne/WallToolPaths.cpp::getInnerContour:779–790` | Returns generated inner contour; captured Native and Rust polygons are exactly equal in both layer calls. No beading repair needed. |
| `PerimeterGenerator.cpp::process_arachne:2503–2515`, especially `:2506` | Infill contours use **member `m_scaled_resolution`**, not the reduced wall-input local; then union and offset2 collapse construct normal fill regions. |
| `ExPolygon.cpp::ExPolygon::simplify_p:223–227` | Simplifies contour/holes with the supplied tolerance before polygon cleanup. This helper receives equal geometry but the wrong Rust tolerance. |

Rust destination is only:
`crates/ares-core/src/project_slice/perimeters/arachne/materialize.rs::materialize_record`,
call to `walls::surface_infill_boundary:241–252`, currently passing
`prelude_record.surface_simplify_resolution` at `:248`.
That shared prelude value intentionally represents reduced wall-input tolerance.
`walls.rs::surface_infill_boundary:281` forwards it to
`geometry::simplify_expolygon_polygons`, propagating the wrong policy for fill.

## Paired diagnostic evidence (read-only GDB)

All runs: outer CPU15, jobs1/test-threads1, independent compilation target,
new Native caches/artifacts for each diagnostic. START/END and script hashes
were posted to inbox under explicit and standing read-only probe approvals.
No inferior writes, set var, function calls, source patches, assertion changes
or comparator changes. Each diagnostic Native full stream equals the frozen
seeded None stream with established generator-only normalization.

1. `/tmp/worker4-arc-paired-eyIt` Native fit trace and
   `/tmp/worker4-arc-rust-memory-BcZy` Rust trace show different pre-fit input:
   105 versus106 points; first difference index58; saved G2 I/J reproduced.
   Thus do not numerically tune ArcFitter to the different input.
2. `/tmp/worker4-fill-input-ssfo`: Native `FillRectilinear::fill_surface_by_lines`
   Bottom surface entry has contour4/hole28; Rust `rectilinear::fill_surface`
   has identical contour4 but hole32. Drift already precedes monotonic connection.
   `FillBase::connect_infill` is not the first divergent handoff.
3. `/tmp/worker4-inner-contour-me15/native-inner.jsonl` and
   `/tmp/worker4-inner-rust-cTHU/ares-inner.jsonl`: both raw generated inner
   contour calls are **ordered-integer identical** (polygons12/32 then4/52).
4. `/tmp/worker4-simplify-input-WZt1/same-geometry-wrong-tolerance.json`:
   Native `ExPolygon::simplify_p` from `process_arachne` and Rust
   `simplify_expolygon_polygons` from `surface_infill_boundary` receive exactly
   identical contour12/hole32 at the first layer, but Native tolerance12000
   scaled vs Rust2400. Backtraces identify the two callers, not merely output
   inference. Native and Rust second-layer matching geometry similarly receives
   12000 versus2400. Stored paired traces preserve all vertices and tolerances.

The source mismatch is now proven: a wall-specific tolerance was reused at an
infill-boundary call whose upstream contract uses the unreduced member value.
Whether correcting that single handoff resolves **all** complete-output bytes
must still be verified after approval; no speculative GREEN claim is made here.

## Minimal implementation scope requested

**Approved implementation adjustment: production allowlist is
`arachne/config.rs` and `arachne/materialize.rs` only**, under the perimeter
boundary above. Preserve `validate_record`'s already computed unreduced
`scaled_resolution` as `ArachneRecordConfig.scaled_resolution`, corresponding to
`PerimeterGenerator.hpp:135`. Pass `config.scaled_resolution` at the infill call;
the operator confirmed this instance name (not ClassicPreludeRecord). Do not
repeat the EPSILON floor/scale calculation in the materializer, or invert the
reduced prelude value. Keep wall-input policy unchanged; no new pipeline stage.

Included: the `process_arachne:2506` argument policy and constructor floor.
Deferred/read-only: ArcFitter/Circle arithmetic, DP/Clipper implementation,
Arachne wall/beading/fuzzy/top-surface regeneration, typed-fill classification,
FillBase/monotonic/grid connectors, placement/centering and GCode replay/parser.
The existing common prelude is a temporary compatibility shell around upstream
flow/geometry concepts; it must not redefine the meaning of the infill member
resolution just because its stored scalar has a convenient name.

W3 centering, W5 interface_shells/classification, W6 grid/Fill files are read-only.
This proposal introduces no region[0], new Unsupported, fallback, arc disable,
geometry substitution, tolerance relaxation, G-code patch or alternate comparer.
The core remains platform-neutral and byte-oriented on all Tier1 platforms.
The counterbore activation three-file allowlist stays separate; its ten old
Arachne reject assertions remain unchanged during this prerequisite.

## RED and verification sequence

Existing frozen RED is
`runner::arachne_counterbore_none::arachne_counterbore_none_primary_complete_output`
on the isolated branch, extracted from the original
`runner::arachne_counterbore::arachne_counterbore_none_primary_complete_output`,
original test commit `9af8025a1`, rebased equivalent `45b1726b4`.
Keep stepped-hole input, stock parent-chain hashes, explicit Arachne/None,
`enable_arc_fitting=1`, resolution0.012, Placement::None, and complete ordered-byte
comparator exactly unchanged. Neither new unit tests nor a new fixture are needed
for this source argument correction; the public E2E already catches the defect.

Failure modes: using 0.2 or dividing a fuzzy-dependent prelude scalar; losing the
EPSILON floor or scale; modifying wall input; introducing a later byte difference;
changing classic controls unnecessarily; and broadening into connector/fitter
code when one scalar is insufficient. If any unresolved parity remains, STOP and
report the next exact mismatch rather than modifying the deferred owners.

After exact SPEC approval:
1. Make the single source-cited argument correction in a distinct production
   commit, after the already frozen RED. No existing assertion changes.
2. Fresh official and seeded directed public None must PASS the entire output,
   not only its first I/J. Both original non-None modes' eight exact guard REDs
   remain expected at this prerequisite stage. Both solid controls and all11
   classic controls must stay PASS, with their recorded raw Ares hashes invariant.
3. Read-only paired fill simplification must now receive12000 for this fixture;
   keep wall input at2400. Evidence is supplemental, not the acceptance seam.
4. Explicitly authorized strict clippy, rustfmt, core/adapter WASM gates; document
   actual scopes and outcomes. Full/workspace run requires separate permission,
   exclusivity and FULL-GO. No push without exact APPROVED.

Exit: source handoff aligned, public None complete output GREEN in both producers,
controls invariant and Tier1 gates accepted. Then request review before returning
to counterbore Bridges/Filled GREEN. Nested-island/multiregion gaps and separate
exact-diff approval for the ten guard assertion changes remain outstanding.
