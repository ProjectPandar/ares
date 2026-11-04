# C2-T repair PLAN amendment: global validation and local Fill admission

Status: proposed PLAN amendment; no production edits.
SPEC C2 `2f2cdfe41`; branch carried onto merged `4c5812d18`.
Only C2-T is active; C2-S / D-M / D-A remain deferred.

## Measured public distinction

Official and seeded, fresh Native roots; the external admission summary
retains complete public inputs,
Native/CLI outputs and wrapper/ELF SHA.

- Local object/model-part `top_surface_pattern=grid`: Native completes,
  54 internal-bridge moves, 42 top moves (rectilinear control 63).
  Merged 4c CLI returns exactly UnsupportedProjectFeature(top_surface_pattern).
- Global `top_surface_pattern=grid`: both Native modes reject before slicing
  with exit 238 (-18), stderr:
  `Param values in 3mf/config error: \ntop_surface_pattern: invalid value grid\n`.
  Merged 4c CLI instead returns UnsupportedProjectFeature(top_surface_pattern).
  The external official and seeded global roots
  retain exact diagnostics and input SHA.

Permanent local E2E active cases reach the same guard at the whole-output
PASS assertion. The first no-candidate slab setup wrongly treated the pinned
binary STL as UTF-8; it is being corrected to scale binary vertex Z only.
Its setup failure is not an admitted RED. All assertions are retained.

## Source boundary

Source `8500fcdccaa10b5099ac20d252af3a7c560046f1`, `src/libslic3r/`:

- `PrintConfig.cpp:10400-10421`: Slic3r::validate checks the four global infill
  pattern option domains before the numeric range loop.
- `Config.hpp:2498-2503`: has_enum_value checks enum_values (the option's
  allowed subset); it does not use the full enum key map.
- `Config.hpp:2084-2092`: local dynamic enum decode uses the complete key map.
- `PrintObject.cpp:3664-3709`: merge local effective region fields without
  enforcing the global role-pattern subsets.
- `Fill/Fill.cpp:858-902,1394-1505`: select Fill by actual surface role and
  generate sparse anchors from grouped Internal fills.

The source has a global validation boundary and a distinct local Fill boundary.
Removing an internal compatibility-shell gate cannot replace the former.

## Why the minimal deletion needs this amendment

Ares' existing `SliceOptions::validate_infill_pattern_options` already ports
these four global domain predicates, and STL loaded validation uses it.
The public 3MF `ProjectSettings::validate_loaded_config` route calls raw
`ProjectSettingsRaw::loaded_range_options` plus retraction validation;
`loaded_range_options` currently invokes numeric/extrusion checks but does
not invoke the existing global pattern helper. Local metadata decoding must
remain separate from this raw global validation.

A top-guard deletion alone would permit global Grid on candidate-producing
3MF input, while Native rejects it. The current Unsupported rejection is
already the wrong error family. Do not preserve that rejection by inventing
a new stage-local whitelist or silently skipping the whole affected region.

## Proposed implementation, after additional public RED

1. Extend the permanent public E2E with global-role rejection controls before
   production edits. Assert Native exact config-error family/body and public
   Ares API/CLI equivalent diagnostics. Preserve raw stdout/stderr/exit/result
   artifacts before asserting, alongside the existing unchanged local tests.
2. In `options/validation/loaded.rs::ProjectSettingsRaw::loaded_range_options`,
   merge `normalized.validate_infill_pattern_options()?` into the existing
   ordered global error map, reusing the ported helper and error renderer.
   This integrates all four upstream global predicates (sparse/top/bottom/
   internal-solid) at their owning raw loaded-config boundary, without new
   parsing, clamping, numeric policy, helper abstraction or fallback.
3. Delete only the top_surface_pattern rejection from
   `project_slice/prepare_infill/bridge_over_infill/transaction/anchor_projection.rs`.
   Leave the local internal-solid gate for its separate C2-S slice.
   Preserve Fill selection, rotation, output order, errors and write-back.

Included scope expansion: the one call-site integration of the already
ported global four-pattern helper, plus top local gate deletion and public
rejection controls. This is upstream validation wiring, not an Ares input
policy. No effective object/volume/range metadata is passed to the global
helper; local Grid stays admitted as measured.

Deferred: numeric range/finiteness changes, D probes, local solid admission,
new patterns/geometry algorithms, layer ranges/profiles, CLI exit redesign.
If existing error rendering or Fill output still differs, retain raw evidence
and stop for a further source-cited PLAN, rather than modifying assertions.

## Exit

Keep corrected local RED and new global-diagnostic RED as separate evidence;
commit all test/assertion expectations before either production change.
Then official and seeded runs,
fresh cache/root each run. Local positive/no-candidate outputs
and global negative diagnostics must match Native, then existing bridge/anchor
regressions plus strict/fmt/wasm32. C2-T lands as its own commit after full
parity.
