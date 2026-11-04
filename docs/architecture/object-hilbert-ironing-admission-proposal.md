# Proposal: Native-reachable object Hilbert ironing admission

Proposal only. The first candidate (grid) was rejected by
source audit before runtime and is NOT proposed. No production/test code or fix
probe. Upstream pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Not already ported

Existing `b9ebd140b` ports **support** Hilbert ironing, NOT object ironing.
Existing `723fc302f` supplies object concentric ironing; later regional ZAA/span
lifecycle changes retain the object pattern guard. Currently,
`fill_entities/ironing.rs:105-110` admits only Rectilinear/Concentric and returns
`UnsupportedProjectFeature("ironing_pattern")` for object Hilbert. This is direct
source evidence that object Hilbert admission is not merged, rather than a log
keyword absence claim. No skirt/brim ByObject, region-speed, or firmware-lift work.

## Minimal input and observed divergence

Input-only frozen 3MFs (`hilbert-active`, `rectilinear-control`,
`hilbert-disabled-control`) with member hashes. No Native output is committed. Target is one closed
10x10x1 mm box,8 vertices/12 consistently oriented triangles, centered110,110,
5 layers at.2mm. No modifiers, no support/raft/tower, no brim/skirt. Preserve
existing source fixture's valid XML namespace declarations; change only its mesh
and project ironing/top-shell settings. All cases share top_shell_layers1.

| Case | ironing_type / pattern | SHA256 |
| --- | --- | --- |
| TARGET | topmost / hilbertcurve | `a741b0504891bd65253418a9a59d949c9e0ab450f2be66085a02a96d9ec60a3d` |
| pattern CONTROL | topmost / rectilinear | `196011bc0790c416c35e54e811ae9db66c437e322a39980ba1df50389b4258e3` |
| disabled CONTROL | no ironing / hilbertcurve | `bb4e77a4ae9ac8c374b130ad3632c27b980d6c6161c4c7db38db93b614378253` |

Controls each differ by ONE project-config field only; all mesh/other ZIP members
are identical. Native FIRST, fresh serial official/seeded roots/caches,
canonical references explicit for Native AND public API/real CLI.

In BOTH modes:

- Native target CLI0,all5 layers have positive model extrusion, one actual
  Ironing block with positive E moves. Not NONE, crash, or rejected input.
- Target Ares validated API returns `UnsupportedProjectFeature("ironing_pattern")`;
  CLI1/no G-code, stderr line1:
  `Error: UnsupportedProjectFeature("ironing_pattern")`.
- BOTH controls accept and PASS COMPLETE unchanged ordered Native/API/CLI
  comparison; raw API==CLI. Pattern control has actual positive Ironing;
  disabled control has none.

**Predicted/confirmed first differing public line:** target CLI stderr line1
above (Native CLI0 with actual G-code). This is an ADMISSION divergence, NOT a
claimed pair of differing G-code byte streams: Ares emits NO target file, so
there is no honest current G-code first-difference index to report. Native
first Ironing marker is line574; first positive ironing move is line578:
`G1 X109.671 Y105.262 E.00002`, after line577 `G1 F900`. These are exact observed
Native reachability anchors in both modes, NOT invented post-fix predictions.
A future RED must require Native equality and actually fail at Ares admission;
never assert Unsupported as passing parity. Any future byte residual after the
port remains STOP; no estimator/M73 relaxation is justified.

## Upstream owning slice and defined-behavior audit

- `PrintConfig.cpp:4298-4308`: ironing_pattern uses the FULL
  `ConfigOptionEnum<InfillPattern>::get_enum_values()` key map. UI advertises only
  rectilinear/concentric, but Hilbert is a recognized serialized enum, not an
  unknown wire coerced to another pattern. Native footer retains hilbertcurve.
- `Fill/Fill.cpp::Layer::make_ironing:1611-1640`: takes config.ironing_pattern,
  instantiates/replaces `Fill::new_from_type(pattern)` and sets bounding box,
  layer_id,Z,overlap. :1705-1731 sets spacing/angle/fixed_angle/link length,
  passes the ironing Surface through fill_surface and appends erIroning paths
  with explicitly computed height/width/mm3_per_mm.
- `Fill/FillBase.cpp:62::Fill::new_from_type`: ipHilbertCurve ->
  **new FillHilbertCurve**, not null/default. `FillPlanePath.hpp:69-77` defines
  this concrete class; `FillPlanePath.cpp:269-275::FillHilbertCurve::generate`
  dispatches to bounded Hilbert generation at :248-266.
- `FillBase.hpp:48-109`: FillParams members have initializers: multiline1,
  extrusion_role0, density initially0 then make_ironing explicitly sets1
  (Fill.cpp:1619-1621). Resolution/anchor lengths and booleans initialized.
  `FillBase.hpp:182-194` initializes base Fill state; make_ironing sets required
  bbox/spacing/angle/layer/Z before consumption.
- `FillPlanePath.cpp:69-164::_fill_surface_single` EXPLICITLY handles density1:
  align=false, snug bounding box, nonzero spacing. The potential optional
  `print_object_config` access at :133-136 is SHORT-CIRCUITED: FillParams
  extrusion_role0 is not erTopSolidInfill, so no null dereference. We do not
  rely on uninitialized flow fields: this plane-path operation generates
  polylines, while make_ironing computes extrusion flow separately.
  For a10mm box at spacing.1, its Hilbert power-of-two domain is small (bounded
  far below integer-shift/size overflow); all mesh vertices/coordinates finite.

Thus this witness is a defined enum/fill branch, NOT a release-only traversal of
a known failing assertion or a UB-based admission gap. Native CLI0 alone was NOT
used as proof: source inputs/initializers/short-circuit consumers were checked.
Contrast rejected grid: `FillRectilinear.cpp:2999` asserts !full_infill while
ironing sets density1; it is deliberately NOT proposed.

## Rust destination

Source boundary is **object `Layer::make_ironing` pattern dispatch**, not a new
ironing pipeline. Destination:
`crates/ares-core/src/project_slice/fill_entities/ironing.rs::append` and its
existing object area/flow/span consumer, reusing the already ported Hilbert
plane-path generator under `fill` and preserving object bounding-box/solid-domain,
layer parity, clipping/ordering and erIroning flow semantics. Existing
rectilinear/concentric branches remain intact; the unsupported guard is a
TEMPORARY compatibility shell to retire only for this cited pattern after public
Native-active RED. Do not blindly delete the whole guard or silently map
to rectilinear. Do not promise all other known InfillPattern values from this
single witness; they remain deferred and must be source-audited separately.

Test-only RED of the frozen target+two controls precedes
ANY production port. Native must positively iron, print all5 model layers and
retain hilbertcurve config; whole ordered comparison and raw API==CLI required
for accepted outputs. Target's admission failure is genuine RED. Independent RED
precedes GREEN. Preserve control
raw API/CLI RED→GREEN, require complete strict target parity or STOP.
No Native-output commits.

Forensic preparation retained separately, never promoted to gate:
initial top_shell_layers0 probes had no Native Ironing (not a witness); enabling
shared top shell produced the reachable gap on original geometry. Initial minimal
mesh serialization lost required-extension XML namespace declarations, so BOTH
Native206/API rejected it BEFORE slicing (not a parity gap). minimal-box-v2
preserves original namespace header and is accepted. A zero-flow rectilinear
localizer was complete PASS, NOT proposed. None of these preparation outcomes
involved production code changes or comparator normalization.
