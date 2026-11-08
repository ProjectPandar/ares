# CLI flush-matrix preparation: palette growth and matrix resize

## Decision and upstream boundary

Continue the source-cited rewrite of configuration preparation already owned by
`crates/ares-core/src/options/flush_matrix.rs` and
`options/flush_matrix/context.rs`. The authoritative upstream revision is
`345efec64c7b95a05ce5cf96df0dde66a158835c`.

This is the pure configuration portion of `Slic3r::CLI::run` before
`Print::apply`, not a new Ares slicing or purge pipeline. It supplies prepared
values to the existing libslic3r-compatible full/runtime configuration views.
The CLI adapter acquires no preparation logic; browser WASM uses the same core
operation and therefore the same prepared bytes.

The registered matrix remains the upstream 4-by-4 default. A singleton zero in
an exported single-filament case is an applied preparation result, not a new
registration default or a serializer convention.

## Owning source contracts

All locations below refer to the authoritative revision, not the older
`8500fcd` line references retained by the original port.

| Upstream owner | Contract |
| --- | --- |
| `OrcaSlicer.cpp:1824-1836` | Capture the old nozzle count and nozzle-volume-type vector before subsequent configuration changes. |
| `OrcaSlicer.cpp:3415-3441` | Determine the new nozzle-volume types; absent CLI types default to Standard per new nozzle. Preserve vector-sensitive old/new comparisons. |
| `OrcaSlicer.cpp:3851-3866` | Distinguish an actual project palette from later typed defaults. A missing or short palette is grown to the loaded logical filament count with `#FFFFFF`, preserving an existing prefix. Palette growth itself triggers matrix preparation, alongside selected CLI colours, an absent matrix, changed nozzle count or changed nozzle-volume-type vector. |
| `OrcaSlicer.cpp:3897-3934` | Use the final project palette for matrix dimensions; retain the support/palette cardinality contract on admitted reconstruction. Do not shrink a longer palette merely to fit a preferred matrix shape. |
| `OrcaSlicer.cpp:3944-3949` | Resize the existing matrix to palette-count squared times new nozzle count, truncating excess entries or zero-filling new entries. Resize multiplier prefixes separately, filling new heads with one. This occurs before dataset-dependent computation. |
| `OrcaSlicer.cpp:3951-4023` | Existing dataset selection and nozzle-major recomputation follow preparation. Source-independent diagonal cells remain zero. This slice preserves the existing calculator and selector boundary rather than changing their algorithms. |
| `libslic3r/GCode.cpp:8244-8272` | Copy the already prepared snapshot, select the existing mode-dependent multiplier, scale and round matching dimensions once, preserve the single-filament mismatch no-op and the existing multi-filament mismatch error. This is not the resize owner. |

A read-only paired-Native trace on the original public witness observed the
16-entry registered matrix immediately before the CLI resize, one logical
filament and one nozzle, then a one-entry matrix containing zero immediately
after it. The Native process completed normally. This locates the producer
before snapshot/export without changing a default, export rule or comparator.

## Rust destination and replacement

`options/flush_matrix.rs::prepare` remains the single existing preparation call
before normalization/materialization in the project effective-config boundary.
Its typed settings are a temporary compatibility representation of the upstream
prepared dynamic configuration, not authority to manufacture a different
pipeline or change static membership.

The present raw-colour early return encodes the old source rule. Replace that
rule with the authoritative palette-growth prerequisite and rebuild predicate.
For a genuinely absent palette, do not treat the registered typed colour as a
supplied prefix. Use the loaded per-filament input count before variant
materialization, not default matrix dimensions, used-tool count or printer name.
Preserve supplied colour prefixes and add white only where the source does.

Within admitted preparation, resize the existing matrix at the source-owned
point. Retain prefix/truncate/zero-fill semantics rather than imposing a final
singleton value or allocating a replacement merely to hide a shape mismatch.
Existing computation remains responsible for its cells after resizing. Existing
unresolved calculator-selector handling is not widened into a new broadcast,
dataset choice or generic map/variant normalization rule.

`PreparationContext` remains short-lived internal old/new preparation context.
Do not put new preparation state into public APIs, persistent project data or
export-only overrides. Re-cite both destination module headers and directly
related preparation comments to the authoritative revision and locations.

The older record `x2d-dual-nozzle-flush-matrix-preparation.md` describes the
`8500fcd` boundary. Its assertion that sparse profiles without raw colours must
never reconstruct is superseded here specifically by the newer palette-growth
branch. Its calculator arithmetic, defined-input selector constraints and
nontrigger preservation contracts are not reopened.

## Included public behavior and failure inventory

Use the existing public byte/profile slicing entry and real Native slicing of
the same resolved profile inputs and original repository geometry. Expected
matrix and multiplier rows come from each runtime Native output, not stored
Native vectors or a second implementation of the resize formula.

- The original no-override direct-STL complete-output witness remains unchanged.
  Its absent palette/matrix must receive the same prepared result as Native.
- Missing and empty palettes must enter the source growth/rebuild branch.
  A supplied matrix does not suppress rebuilding when palette growth triggers it.
- A short supplied palette preserves its existing prefix and grows white entries
  for the loaded filaments; multi-filament prepared output remains source-owned.
- With a complete palette and no other trigger, a supplied valid matrix is
  retained. Default, Saving and Fast export behavior and rounding are observed
  through the unchanged public output, not implemented again in preparation.
- A supplied single-filament mismatched matrix with no trigger remains subject
  to the existing exporter no-op, not unconditional matrix repair.
- Existing fully specified multi-filament/multi-head reconstruction, multiplier
  prefix, supplied custom-matrix and malformed nontrigger controls remain intact.
- Existing membership controls retain their public kinds and applied-value
  behavior. Boolean preset defaults retain their canonical wire representation.

Keep complete Native/Ares payloads, inputs, commands, producer identity and
repeatable comparison artifacts. The established complete ordered-byte
comparison defines stream equality; matrix-row agreement alone does not imply
whole-output parity.

## Deferred and prohibited changes

No registration/default/membership edit, footer blacklist, serializer omission,
comparator relaxation, missing-Native-policy change or forced singleton output.
No CLI-adapter preparation, filesystem/UI dependency in core, new crate or public
API, GUI thumbnail work, calculator/predictor math change, new dataset fallback,
broad map/variant normalization, wipe-tower, purge-body, motion or geometry work.
