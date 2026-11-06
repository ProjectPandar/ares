# Pre-cooling filament variant materialization

## Owning upstream boundary

Pinned OrcaSlicer beta: `345efec64c7b95a05ce5cf96df0dde66a158835c`.
This slice ports two nullable integer option members and their existing static
filament-variant materialization, not a new pre-cooling or ramming algorithm.

- `src/libslic3r/PrintConfig.cpp:8879–8886` registers
  `filament_pre_cooling_temperature` as coInts, nullable, default
  `ConfigOptionIntsNullable{0}`. Zero means disabled; nil is not numeric zero.
- `PrintConfig.cpp:8908–8916` registers
  `filament_pre_cooling_temperature_nc` with the same type/default. Its separate
  hotend-change meaning is retained; it is not an alias of the first key.
- `src/libslic3r/PrintConfig.hpp:1926,1929` declares both static GCodeConfig
  members as `ConfigOptionIntsNullable`. These are not raw export-only strings.
- `PrintConfig.cpp:10154–10159` lists both in
  `filament_options_with_variant`, adjacent to separate ramming options. The
  generic load-time singleton/absent expansion at `PrintConfig.cpp:11431–11457`
  and its callers remain unchanged in this slice.
- `PrintApply.cpp:1436–1455` performs static filament gathering before runtime
  config differences; `Print.cpp:4723–4755` repeats it from the saved checkpoint
  on static map writeback. Both Rust entry routes must share the same field
  materializer.
- `PrintConfig.cpp:11884–11948` owns source-index resolution;
  `11965–11981` dispatches registered vectors, including coInts, through
  `gather_option_values` at11862–11882. Its exact payload clone preserves nil,
  explicit zero and nonzero integer values, repeats the original front for a
  missing source column and leaves empty sources unchanged.
- `GCode.cpp:8244–8246,8342–8353::append_full_config` serializes the materialized
  full config and omits an all-nil option. The normal serializer owns mixed nil
  token spelling; the footer is not a separate vector-expansion stage.

## Destination

At the approved pre-cooling starting point, both keys are already registered
as `IntsNullable`, default0, in
`options/registry/definitions/table/pre_middle_filament.rs`. Their typed owner
is absent from `FilamentGCodeSourceOptions` and therefore from
`project_variants/filament.rs::materialize`.

The current `options/config_export/full_config.rs` raw-settings/default path
can emit these registered keys, but cannot represent their resolved logical
filament vectors. This raw echo is only a temporary compatibility shell around
the upstream members, not the long-term source of applied config values.

Generic load-time singleton/absent filament variant layout follows
`PrintConfig.cpp:11431–11457` and `OrcaSlicer.cpp:4250–4259`.
This slice must not modify that implementation or `profiles/project_refresh/keys.rs`.
Add exactly these two keys to `options/registry/key_lists.rs`'s
`FILAMENT_OPTIONS_WITH_VARIANT_KEYS` in the same implementation commit as their
shared gather assignments. This connects the members to the existing load-time
default completion and filament variant layout without changing its algorithm. Promote
exactly these two existing registered options into
`options/filament_options/gcode_source.rs::FilamentGCodeSourceOptions` as
`Vec<Nullable<OrcaInt>>`, initialized using the existing `nullable_ints(&[0])`
helper. Use the existing option-group decoder/builder and append behavior;
register the two fields in the existing declaration order, standalone GCode
source wire serializer and filament wire serializer. Reuse
`serialize_nullable_vector` exactly as for the neighboring `filament_flush_temp`.
Do not introduce another nullable representation or token parser.

In `options/project_variants/filament.rs::materialize`, route both vectors
through the existing `select_vec_field!` / `select_filament<T: Clone>` helper.
Use the existing source-index list, checkpoint/full/runtime views and static
map-refresh lifecycle. Do not create a pre-cooling selector, index formula,
post-render duplication, exporter override or registry regeneration.

The typed entries must take precedence through the existing full-config
collector. Existing all-nil handling must prevent raw/default resurrection.
No special-case rendered-token edit is permitted. If promotion exposes a
shared serializer or lifecycle defect, stop and bring its owning source
boundary back for review rather than patching a token for these two keys.

This slice does not redesign profile inheritance. Any directly necessary field
wiring must use the existing nullable integer/profile-vector machinery and
retain the registered type/default, without changing generic profile rules.
Keep touched modules below400 LOC; split only if the new field wiring actually
crosses that limit. Core remains platform-neutral on browser WASM, Windows,
macOS and Linux, without filesystem or terminal behavior.

## Included and deferred

Included: two typed nullable integer option members, their normal decode/wire
ownership, complete static logical-filament vectors at initial materialization
and static map refresh, exact source-index/order/front-on-miss and empty-source
behavior, nil/zero/nonzero identity and normal full-config serialization.
For absent authored values, Native materializes both registered defaults as
`0,0` for two logical filaments, while the raw Ares path emits singleton `0`.

Deferred: the neighboring ramming speed/time, nozzle-change retract and preheat
options; other missing variant-key cohorts; actual pre-cooling/fan/ramming
command generation; new multi-nozzle/Hybrid or dynamic grouping producers;
sequential collection; GUI/plate read-back; registry regeneration; machine
writer, motion, temperature-reader or per-layer algorithm changes. This is a
source-cited member/materializer rewrite, not an Ares-owned cooling pipeline.

## Tests

`crates/ares-cli/tests/orca_parity/option_coverage/pre_cooling_variant.rs`
asserts Native replay stability, successful public API/CLI slicing, raw API/CLI
output equality, labelled Native/API/CLI footer equality and positive tool1
model activity with High Flow source columns1/3 selected:

1. `pre_cooling_variant_absent_defaults`: absent authored values produce two
   logical zero entries for both keys.
2. `pre_cooling_variant_distinct_columns`: independent four-column integer
   vectors produce the values at source columns1/3 in logical filament order,
   independently of the single physical nozzle.
3. `pre_cooling_variant_mixed_nil_zero`: source vectors `[nil,0,123]` and
   `[0,nil,173]` produce `[0,nil]` and `[nil,0]`. The three-entry shape bypasses
   singleton/per-filament load normalization (`PrintConfig.cpp:11431–11457`)
   and asserts original-front fallback for missing source column3 without
   changing nullable serialization.
4. `pre_cooling_variant_all_nil_absence`: an all-nil source omits its key,
   while the other explicit-zero source remains a zero vector; the omitted
   key is not resurrected from raw/default settings.

## Failure modes

1. Raw source vectors leak into applied output instead of logical vectors.
2. Nullable integer members become nonnullable.
3. Explicit zero is replaced with nil or a default.
4. Nil is replaced with numeric zero.
5. Source indices or logical filament order are incorrect.
6. Missing source columns do not use the original front value.
7. Empty sources overwrite the destination.
8. Unused logical filament entries are lost.
9. All-nil options are resurrected or typed/raw keys are duplicated.
10. Public API and CLI output diverge.
