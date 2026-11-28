# Slice a lone belt purge prism when no filament changes

This extends the no-change branch of [active belt purge](belt-active-purge.md)
to a plate with one marked object and no separate object to align against.

## OrcaSlicer boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`):

- `src/libslic3r/BeltPurge.cpp:48–61`, `Print::has_belt_purge_tower`:
  active belt purge requires belt mode, its own enable option, more than
  one `filament_diameter` entry, and a marked object, with spiral mode off.
  It does not require a second object or a scheduled filament change.
- `src/libslic3r/Print.cpp:3614–3615`, `Print::process`: the active belt
  planner runs for layer-by-layer printing.
- `BeltPurge.cpp:109–123`, `Print::_align_belt_purge_layers`: find the tallest
  non-prism object with layers. If there is none, return without shifting
  the prism or treating the missing reference as an error.
- `BeltPurge.cpp:146–168`, `Print::_plan_belt_purge`: clear old purge data,
  call alignment, then build unprimed tool ordering. Empty ordering is an
  error; a missing alignment reference on its own is not.
- `BeltPurge.cpp:170–183`: scan the ordering for an actual filament change.
  With none, return before allocating flush, shortening the prism, or
  changing its fill. The printer still emits the prism's ordinary paths.

## Ares mismatch and destination

On a translated, single-part, single-instance belt plate with two loaded
filaments, one used filament, and no plate events, Native emits all 42
layers with the purge flag enabled and the prism marked. There are no
swaps, classic tower paths, or purge-completion comments. Ares refuses the
same input before emitting G-code. With just the purge flag disabled, both
CLIs emit byte-identical G-code after generator-banner normalization, so
this fixture does not require a separate motion or formatting change.

`crates/ares-core/src/project_slice/belt.rs` (`admit`,
`admits_multiple_filaments`) currently assumes active purge must pair the
marked prism with an ordinary object. Its material gate also assumes the
object checked in active purge is the ordinary one. Replace that assumption
for the following narrow shape only: one printable instance of the lone
marked normal part, one physical nozzle, at least two configured logical
filaments, exactly one used logical filament, no plate events, and no
support or raft. Use the existing resolved filament-usage result, including
role and range selectors, instead of assuming two configured slots imply
a change. Check material restrictions against the actual printed object;
keep the existing single-nozzle, unmixed, and unpainted restrictions.

`project_slice/belt/active_purge.rs::align_active_purge_plans` already returns
without shifting when it finds no reference. The existing
`project_slice/gcode_emit/tool_ordering/belt_purge.rs::ToolOrdering::plan_belt_purge`
keeps its empty-print error for empty ordering and returns successfully
without purge edits when the ordered tools never change.
Reuse these branches and the existing brim exclusion for the marked prism.
Do not invent a reference grid, hide the marker, disable active purge in the
resolved settings, or emit compensating tool changes or purge G-code.

## Exclusions

Keep the admitted prism-plus-model route unchanged. A lone prism with a
scheduled change or with plate events remains outside this slice. Do not
add support, raft, multi-instance geometry, custom or variable layer-grid
handling, mixing/painted materials, classic tower behavior, or GUI prism
mesh generation. This slice does not change purge-volume accounting or
missing-absorber warnings.

## Verification

Generate the marked one-object, two-slot, one-used-filament fixture from
the repository's synthetic belt fixture helpers. Replace the no-event arm
of the deferred-prism rejection check with live ordered G-code parity; keep
the existing event-bearing input outside admission. Use the same CLI argv
for Native and Ares, and check the core slice API with the existing
CLI/API parity helper. Require actual printable extrusion, no scheduled
swap, no classic tower or purge-completion marker, and ordered whole-file
parity after the existing generator normalization. Do not add a mask or
commit a Native output. The purge-disabled marked control continues to
pass unchanged.

Prove the new parity case fails on the test-only commit at the Ares refusal
in both Official and Seeded, then passes in both lanes on the implementation
tip. Re-run the existing no-change, active purge, multi-filament belt, and
purge-disabled coverage; use paired scoped base/tip runs of the belt suites
with no passing-to-failing cases and no selected skips. Run the repository's
native and WASM static checks at the exact tip.
