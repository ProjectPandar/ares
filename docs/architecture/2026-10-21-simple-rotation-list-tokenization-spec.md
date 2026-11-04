# Simple rotation lists: comma items are not whitespace-separated angles

Docs-only SPEC following the source-cited discovery, followed by a separate
five-input RED from main `3fbad615e334a283596d0e6acff921f1d02804c6`. No GREEN.
The aligned chain758b is unchanged.

## Owning upstream slice and exact Rust boundary

Pinned OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `src/libslic3r/Fill/Fill.cpp:206-210`: non-meta rotation templates delegate
  to `ConfigOptionFloats::deserialize`, then index cyclically by actual layer.
- `src/libslic3r/Config.hpp:868-890`,
  `ConfigOptionFloatsTempl<false>::deserialize`: split only by comma using
  `std::getline(..., ',')` at879, trim each item, and read its numeric prefix
  with `std::istringstream >> double`. Thus `45 90` is one item yielding45.
- `Fill.cpp:63-68`: the distinct meta-language branch uses regex `[\s,]+`.
  Its delimiter must not be applied to the simple-list branch.

The sole future Rust destination is
`ares-core/src/project_slice/group_fills/params/projection/rotation.rs::simple_rotation_angle`
(lines47-63 at the base), limited to **simple-list tokenization/trim**
and correction of its misleading comment attributing `[\s,]+` to upstream
ConfigOptionFloats. This existing adapter is a temporary compatibility shell
around the cited upstream interpreter, not an Ares-owned parser/pipeline goal.
No new helper/API/configuration or generic ConfigOptionFloats rewrite.

Included: comma-item and ordinary ASCII whitespace handling demonstrated by
these inputs, retaining current comma/single-angle behavior. Deferred: meta
interpreter/tokenizer, shared numeric helper behavior, malformed/empty-item
semantics, hex/inf/nan, locale/Unicode or other numeric grammar, fixed_angle
projection/grouping, filler classes, geometry, Flow and emission. Do not touch
`rotation/meta.rs` or redesign shared number parsing to address this witness.
No fallback, unsupported guard, tolerance or output normalization expansion.

## GUI and source witness

`PrintConfig.cpp:3993-4004` defines advanced Strength `coString` sparse rotation
template; `GUI/Tab.cpp:2474` exposes it. `Config.hpp:895-900` validates digits,
commas and `isspace`; `GUI/Field.cpp:524-549` retains the original validated
string. `45 90` therefore survives ordinary GUI editing unchanged, despite the
comma-separated tooltip. `Tab.cpp:1769-1798` accepts Rectilinear without the
unsafe-pattern warning or string rewriting.

Read-only actual layer5/7 return observations: Native
`calculate_infill_rotation_angle("45 90")` yields0.7853981633974483 radians
(45 degrees), while Rust simple helper yields90 degrees. For `45,90`, both
resolve90 degrees at those same actual layers; layer6 resolves45 on both sides.
Units and actual layer arguments were captured, not inferred from event counts.
Four diagnostic whole streams equal canonical streams. Vector cardinalities
are source-derived, not claimed captured live layouts.

The first strict target difference is **M73-related**, not a later-motion-only
claim: byte1307/line58, Native `;_SET_FAN_SPEED_CHANGING_LAYER`, API/CLI
`M73 P4 R14`; normalized lengths216887/209109. Do not filter progress, times,
F, E, wipe/retraction or footer. The boundary is interpretation, not a cooling
or emission repair. The discovery does not guarantee future GREEN closure.

## Frozen five-case RED and public seam

Reuse the five exact discovery3MF bytes as inputs only:

| Case | Template | Discovery both modes |
|---|---|---|
|space-list|`45 90`|strict DIVERGENT|
|tab-list|`45\t90`|strict DIVERGENT|
|comma-list|`45,90`|complete PASS|
|single-angle|`45`|complete PASS|
|meta-space-control|`+45 +90`|complete PASS|

Meta-space is a parser-boundary control: GUI editing meta spelling may
canonicalize separators through the other Field.cpp branch. It is not claimed
as the primary literal GUI-authored witness. Preserve this qualification; its
purpose is to freeze the distinct meta behavior, not broaden this slice.

All five derive from the same existing Native-exported authored overhang input
for `infill_rotate_template::t24_sparse_list_control`. Classic walls,
Rectilinear sparse, multiline1, monotonic solid surfaces; common GUI settings
turn brim/skirt off. Between cases the sole variable is
`sparse_infill_rotate_template`; geometry/transforms/non-settings ZIP members
remain exact. No painted, aligned or Arachne wall-generator behavior is a
proposed change. No mesh/position/density tuning.

Use five thin unconditional direct calls to unchanged
`native_stream_case::check` / shared NativeCase. Freeze input SHAs and source
citations; no Native numerical/G-code constants, output goldens, copied
runner/normalizer, inverted assertions or error-as-success tests. The shared
lifecycle generates Native twice first, then validated public API and real CLI,
with the existing complete ordered-byte generator/date-only comparison.
Targets must fail normally; independently inspect retained API/CLI bytes when
strict comparison panics before the wrapper's trailing metadata/assertions.
Do not claim an unexecuted adapter/tail assertion was reached.

## Stage exits and immutable boundaries

1. Commit this SPEC alone from3fbad, then separately commit the five input3MFs,
   README, thin tests and registration. Production/core, shared helpers,
   existing tests/assertions, existing inputs and comparison policy stay exact
   base. No ignored/allow/expect or feature-guard admission changes.
2. On exact RED tip, official and seeded each show two intended strict FAIL and
   three PASS. Fresh Native double replay first, stable hashes/repeats, actual
   public API/CLI whole-output results and independently checked raw equality.
   Preserve actual binary/command/status/input/output evidence without fictional
   cache receipts. No implementation in this stage.
3. Run fmt and locked workspace/all-target Clippy-Dwarnings after test edits,
   with fresh per-mode caches/artifacts.

All core changes remain future-only. Tier1 WASM/browser, Windows, macOS and
Linux boundaries are unchanged; this is source-owned string interpretation,
not filesystem/UI/terminal logic entering core.
