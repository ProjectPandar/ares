# Original project keys and late FullPrintConfig default completion

## Upstream boundary

Pinned OrcaSlicer source: `345efec64c7b95a05ce5cf96df0dde66a158835c`.
This slice rewrites the separation between authored project option membership,
filament preset defaults and late full print defaults. It does not introduce a
new configuration pipeline or extend filament variant selection.

- `src/OrcaSlicer.cpp:2787–2791` captures `project_keys` before preset refresh,
  with the existing `filament_extruder_variant` exception.
- `OrcaSlicer.cpp:3575–3577,3606` uses that original set when refreshing a
  project without a changed-settings list. Options created later are not
  retrospectively authored project keys.
- `OrcaSlicer.cpp:3682–3704,3743–3763` restores kept variant values or applies
  refreshed values. Those replacement and source-index operations are not
  changed by this slice.
- `OrcaSlicer.cpp:4250–4259` constructs `FullPrintConfig` after refresh and CLI
  overrides, applies the effective dynamic config into it, applies it back,
  then calls `normalize_filament_values_to_variants`.
- `libslic3r/PrintConfig.cpp:8928–8937` defines nullable
  `filament_retract_length_nc` with default `10`.
- `PresetBundle.cpp:508–518` separately nils nullable extruder override keys
  in the default filament preset. That collection default is not the full
  print default source.
- `Preset.cpp:323–331`, particularly the condition at325, excludes filament
  extruder override keys from all-nil reset to inherited defaults. Explicit
  NC nil remains off; it must not become `10` through this correction.
- `PrintConfig.cpp:11431–11457` owns singleton/per-filament variant layout.
  Existing selected/front gathering at11862–11994 is unchanged.

The existing raw completion helper is a temporary compatibility shell around
late full-config application. Calling a preset-default provider from it and
persisting its synthetic keys as original project membership do not match
these upstream boundaries.

## Destination

Production changes are limited to:

1. `crates/ares-core/src/project/load.rs`: retain pre-completion raw settings
   as the project membership source. Complete the already-required typed
   deserialization copy instead of changing the raw stored in documents.
2. `crates/ares-core/src/project/domain/preset_refresh.rs`: retain the same
   distinction when rebuilding settings after refresh. Consume the existing
   deserialization copy rather than making another copy.
3. `crates/ares-core/src/project/raw_settings/filament_variants.rs`: complete
   missing options using FullPrintConfig default membership and values,
   then reuse existing wire decoding and variant layout. Do not use the
   default filament preset's nil policy for this operation.

No new raw or mesh copy is permitted. Both callers already require one raw
copy for typed deserialization; that copy is the materialization target and
is consumed by the builder. Original presence remains in the existing raw
representation, not a new provenance table or second configuration model.
The core boundary stays pure and portable across browser WASM, Windows,
macOS and Linux, without filesystem, process, terminal or UI dependencies.

## Included and deferred

Included: preserve absent versus present project keys across default
materialization; refresh an absent key from its supplied system filament;
keep an authored or protected value according to existing refresh rules;
use full print defaults only for missing eligible options; preserve explicit
nil, defined zero, negative sentinels and fractional values; reuse existing
layout and the existing raw-copy budget.

Do not reset a present all-nil vector, change the default filament preset,
change registry definitions, or add an exception for a particular key. A
missing NC key can receive the full print default `10`, a supplied system
filament can provide `14`, and an explicitly nil NC key remains nil. These
are distinct source states, not interchangeable fallback values.

Deferred: gather helpers, index and activation formulas, profile inheritance,
catalog resolution/includes, CLI algorithms, serializers, comparators,
normalization policies beyond this default-completion boundary, new options,
physical retraction or heating behavior, geometry and motion generation.
No new ownership or generalized change-tracking mechanism is introduced.

## Dependency and test order

The six nullable float members must already have typed/wire representation,
sorted variant-list registration and ordinary gather assignments before the
public footer regression can distinguish completion from missing fields.
Therefore this slice's SPEC and RED follow that materialization port. This
correction is a prerequisite for combined behavioral qualification, not for
the Rust field declarations themselves. The materialization port alone does
not satisfy the complete default/refresh contract.

Add the public X1C regression after those six keys are represented, observe
RED, and review this independent slice before changing its three production
destinations. Existing six-key tests and their inputs remain unchanged.

## Tests

The public seam is unchanged project bytes through the pinned Native slicer,
validated core slicing API and CLI, with the same supplied preset catalog.
Use the existing unmodified
`tests/parity/avoid-crossing-external-frame/x1c-detour0.3mf` input. Its project
omits preheat and NC retract keys and has empty changed-settings columns.
Its system filament includes the dual-variant template with preheat `10` and
NC retract `14`; the selected Standard variant must retain those values.

The new test asserts the complete Native six-key cohort before comparing
API/CLI: preheat `10`, ramming travel `0`, travel NC `0`, ramming speed `-1`,
speed NC `-1`, retract NC `14`. Preheat and NC retract are the direct
presence/completion assertions; the other four are unchanged controls through
the same boundary. Also assert the selected variant and self index, Native
repeatability, successful API/CLI slicing and raw API/CLI equality. Missing
lines must be represented explicitly so RED identifies the cohort mismatch,
not an unqualified missing-line unwrap. Retain separate unchanged strict
whole-stream comparisons; do not replace them with a new normalization rule.

The original Ender/MK4 default cases must retain `10` at their logical widths.
Existing fractional, ragged nil/zero/negative-sentinel and all-nil omission
cases must remain unchanged, as must the original refresh/pre-cooling/ramming
controls. Full-config completion preserves the pinned Native option values
and variant layout without introducing unrelated output or body changes.

## Failure modes

1. Completing a missing key early makes it appear authored, preventing a
   refresh from replacing a synthesized default with the system preset value.
2. Full print completion uses the filament preset nil default instead of the
   source full print default, omitting a required NC retract vector.
3. Resetting NC nil to `10` destroys the source's explicit off state.
4. A copied completed map is persisted as original project membership, or a
   new presence table duplicates state rather than retaining the existing raw.
5. A second raw copy, whole archive copy or mesh copy increases allocation
   cost instead of consuming the existing typed-deserialization copy.
6. A defined zero, negative sentinel or fractional value is treated as absent,
   or a default replaces a supplied value.
7. A fix changes gather/index/activation, inheritance, catalog, CLI, registry
   policy or comparator behavior rather than the specified completion seam.
8. Missing footer lines cause an unqualified RED before Native premises or
   API/CLI success and equality have been established.
9. Focused footer success masks a whole-stream or official-inventory regression.
