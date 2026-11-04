# Avoid-crossing safe-zone offset: upstream default miter limit

## Owning upstream boundary and exact Rust destination

Rewrite the offset-parameter choice in
`OrcaSlicer/src/libslic3r/GCode/AvoidCrossingPerimeters.cpp`,
`AvoidCrossingPerimeters::init_layer` lines 1324–1328:
`offset_ex(layer.lslices, -get_external_perimeter_width(layer) * coeff)`.
The omitted arguments resolve through `ClipperUtils.hpp::offset_ex` (346) to
`DefaultJoinType = jtMiter` (19) and `DefaultMiterLimit = 3` (27).

Exact Rust destination:
`crates/ares-core/src/project_slice/gcode_emit/motion/path/avoid_crossing/safe_zone.rs`,
its private `MITER_LIMIT` consumed only by `build`'s `offset_expolygons` call.
The planned production delta changes this one offset parameter from 2 to 3.
The existing safe-zone carrier is a temporary adapter around the upstream
`m_lslices_offset` concept, not an independently designed Ares routing pipeline.
No public API, configuration, filesystem, terminal or platform-specific core
behavior is added.

## Required read-only precondition: proved before this spec

At the implicated print Z `0.6000000000000001`, GDB captured Native
`layer.lslices` and Rust `safe_zone::build`'s `geometry.layer_slices` for both
thin-gap targets. Complete arrays match in their original order: every
polygon, contour, hole and integer vertex. Each target has two contours and
three holes. No canonicalization, sorting, tolerance or rounding was used.
Rust was observed twice per target with the same inputs.

Evidence (thin pre-offset lslices equality) is retained externally.
SHA256 of compact ordered coordinate arrays:

- thin-auto-gap: `e0f21ef7a2eaede94ff9d969a668b8f0c4e72351e50c6b30dc61ecefb569dde5`
- thin-percent-gap: `742ce651f704ced5727edbfca1c663c344f93cbfa4e542398a978f55a25be0ec`

Captured on `e6e5ca55d4d16a2d61a925eb05cf3c0fef219fca`, after the
endpoint-contact and inward-displacement slices. Those two slices
remain unchanged; the original WIP branch is untouched.

## Observed divergence and bounded hypothesis

Stable Native double replay and the real CLI still disagree on the two
thin-gap frozen inputs. Thin-auto's first difference is F1496 versus F1495;
thin-percent first differs in travel. These are not admitted as independent
cooling or width-resolution bugs.

On thin-auto's first implicated post-wipe travel, Native has a seven-point
route. Rust `safe_zone::contains` returns true before and after the wipe and
returns a straight route without entering the boundary router. Native's
safe-zone corner contains `[-1769990, 541813]`; Rust instead has the bevel
vertices `[-1769990, 70383]` and `[-1436639, 208462]`. Pre-offset inputs are
exactly equal, while the offset's miter parameter explicitly differs from
upstream. The read-only finding is retained externally.

Hypothesis to test after this spec: restoring the source default
miter limit restores safe-zone containment/routing and its downstream timing
consequences. Whole-output GREEN for both targets is required; a smaller
feedrate or travel difference is not completion.

## Included and deferred behavior

Included: jtMiter's existing safe-zone offset with upstream miter limit 3.
Keep the 0.6/0.5/0.45 retry sequence, external-width selection, current scaled
arithmetic, containment predicate, empty handling and error propagation as-is.
No special case for thin input names, width modes, tools or region counts.

Explicitly deferred: **`avoid_crossing/boundary.rs` retains its miter limit 2**.
It has a different source/consumer boundary and needs an independent witness;
this spec does not permit a search-and-replace of miter constants.
Also deferred: interlocking thin morphology, nozzle/flow resolution,
intersection predicates, writer state, inward cast order, cooling arithmetic,
normalization and new Unsupported/fallback behavior. If the one-parameter
change leaves a residual, stop and report it without broadening this slice.

## Native-first public matrix, shared harness and failure inventory

Confirmed seams remain Native CLI twice, validated byte-in/options core API,
and real Ares CLI. Reuse `runner/native_stream_case.rs` rather than copy its
assertions. Reuse archives already on the stack. Bring the two
input-only targets and the missing three-region beam1 control unchanged from
the WIP branch; do not copy already-present control archives.
No Native output is committed.

Each case below runs in **both official and seeded modes**, preserving the
existing complete ordered generator-only Native/API and Native/CLI comparisons,
Native double-replay stability, frozen SHA checks and API/CLI raw equality.

| Case | Contract | Input SHA256 |
| --- | --- | --- |
| thin-auto-gap-beam1 | real RED target | `a9cf6ad1fbef7df90556814c70791f8998a9c230c50a040cbde35381bee35e80` |
| thin-percent-gap-beam1 | real RED target | `56144334ff023960cc36dfa5b59d936c9b88d3dd772917dd09808d88d1f925a4` |
| odd-kernel-beam1 | PASS control after endpoint slice | `180d44d516f560510bb19a6584e6f41cd092cb16788a6a4091d7a32c69d841b7` |
| orthogonal-grid-beam1 | PASS control after inward slice | `afed0f4f4417916d2963363754bc45ef1300f6bbc2d6dfbcc9d52cf751a0c312` |
| three-region-beam1 | PASS control on clean generator | `71a5035e7cdade9b98ecd96379fdee03ced8151dc54958e959e5b444f66e6701` |

Three-region is already strictly equal on the clean generator, not a new RED
or a behavior fix claimed here. Its input may be added unchanged if it is not
yet present on the stack; do not duplicate its beam0 control.

Failure modes: a beveled hole corner changes safe containment; an unnecessary
straight travel replaces Native's detour; route-length changes perturb cooling
feedrates; a wider miter regresses existing odd/orthogonal/three-region streams;
a local improvement leaves later output different. Assert whole outputs, not
just the first changed line. Preserve launch/input/output/comparison artifacts.
No ignores, test skips, new allow/expect, probes, altered comparators or silent
fallbacks may be used to satisfy the matrix.

## Sequence and integration gates

This is a **docs-only SPEC**, based on
the endpoint + inward stack. It does not advance either branch reference.

1. Land this spec and its pre-offset equality evidence first.
2. Then commit fresh runtime Native-first RED for the
   two targets and three PASS controls, in both modes, using shared runner.
3. Production-only GREEN may change only the safe-zone offset miter parameter.
   Both targets and all controls must wholly pass in both modes; otherwise stop.
4. Changed files stay below 400 lines; fmt and workspace/all-target clippy
   with `-D warnings` must be clean; preserve browser WASM/native separation.
5. Integration must
   rebase onto then-current main and pass exact-final-hash official and
   seeded full parity with 0 FAIL/0 skipped. This spec
   grants no earlier-baseline exemption.
6. Keep all Native/GDB evidence outside Git.
