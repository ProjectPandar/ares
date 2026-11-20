# First-layer travel jerk selection

## Upstream boundary

Port the first-layer jerk arm of `Slic3r::GCode::travel_to` from OrcaSlicer
v2.5.0, revision `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`:

- `src/libslic3r/GCode.cpp:10519-10531`: initialize jerk to zero; on the
  first layer resolve `initial_layer_travel_jerk` at
  `get_nozzle_config_index(m_writer.filament()->id())`; select it only when
  both nozzle-configured `default_jerk` and the resolved value are positive.
- `GCode.cpp:10532-10558`: ordinary travel and short-travel jerk selection
  belong to the non-first-layer arm, not a fallback for a zero initial value.
- `GCode.cpp:10561-10565`: pass the selected value to the existing merged
  Klipper writer or separate travel-acceleration/XY-jerk writer.
- `src/libslic3r/GCode.hpp:991`: `on_first_layer` requires a current layer,
  layer id zero, and bottom Z within `EPSILON` of zero.
- `src/libslic3r/PrintConfig.cpp:4039-4047`: the nullable float-or-percent
  vector defaults to `100%`; `ratio_over` is `travel_jerk`, not `default_jerk`.
- `src/libslic3r/Config.cpp:715-738`: `get_abs_value_at` resolves the initial
  value and its travel-jerk reference at the same nozzle index.
- `src/libslic3r/GCodeWriter.cpp:483-532,535-577`: preserve existing flavor,
  machine-limit, threshold and deduplication behavior. Zero requests do not
  request a zero-jerk command or update the cached jerk.

This is an upstream rewrite slice, not a new motion pipeline or writer policy.

## Rust destination and decisions

The destination is the existing motion emitter:

1. `crates/ares-core/src/project_slice/gcode_emit/motion/options.rs` gains
   one resolved `f64` initial-layer travel-jerk field alongside travel jerk.
2. `motion/options/build/fields/assemble.rs` resolves the existing
   `gcode.initial_layer_travel_jerk` vector with `absolute` at the existing
   filament-to-nozzle index, using the same effective object/process
   `travel_jerk` cell that already supplies ordinary travel jerk. Reuse the
   existing object override and nozzle-resolution mechanisms.
3. `motion/state.rs::begin_path_travel` selects this field in its first-layer
   jerk arm instead of unconditionally selecting ordinary travel jerk. Use
   the existing `EmitState::on_first_layer` predicate, which carries the
   upstream layer-id/bottom-Z condition. Preserve the default-jerk positive
   gate; a nonpositive resolved initial value supplies zero, not ordinary
   travel jerk. Do not truncate or round this double-valued jerk.

The current first-layer `travel_jerk` assignment is a temporary compatibility
shell for the cited `travel_to` arm and is replaced, not retained as a fallback.
The nullable vector, configuration registry, public byte-oriented API, and
`options/acceleration.rs::parse_jerk_options` already exist; the latter already
uses travel jerk as the percent reference. Do not introduce another parser,
public option, configuration representation, or compatibility layer.
`src/speeds/config.rs` is not the owning emission boundary and needs no change.
The core remains platform-neutral with no filesystem or terminal behavior.

## Nullable resolution

`ConfigOptionFloatsOrPercentsTempl::nil_value` stores `{NaN, false}`
(`Config.hpp:1424`); its inherited `get_at` selects the indexed entry or the
first entry, without replacing nil (`Config.hpp:676-681`).
`ConfigBase::get_abs_value_at` has no nil-specific branch: it invokes
`FloatOrPercent::get_abs_value`, which returns that non-percent NaN unchanged
(`Config.hpp:47-55`, `Config.cpp:715-738`). The travel selector's positive
comparison consequently fails if an unresolved nil reaches it. This is not
an ordinary-travel fallback.

Rust's existing `OrcaFloatOrPercentsNullable::get_at` already represents nil
as `FloatOrPercent::Float(f64::NAN)` (`options/config_types/vector.rs:114-119`).
Reuse that getter and the upstream positive-value comparison; add no special
nil handling. Public-project nil normalization/inheritance and end-to-end nil
parity are deferred because the projects used for verification contain explicit values,
not nil. No unqualified input-loader or nil-resolution policy is introduced.

## Included and deferred behavior

Included: first-layer travel jerk's nozzle-indexed absolute/percentage
resolution, positive-value selection, zero suppression through the existing
writer, and preservation of subsequent-layer selection.

Deferred and unchanged: first-layer travel acceleration and its unsigned local;
non-first-layer role/short-distance acceleration and jerk policy; physical
filament retraction thresholds; post-start temperature waits; writer precision,
clamping, thresholds and cache policy; extrusion jerk, lift, raft generation,
viewer/processor data, and the broader configuration migration. The existing
first-layer predicate is reused, not redesigned. Exhaustive multi-nozzle and
raft geometry parity is not claimed by single-nozzle synthetic qualification.

For finite non-nil acceleration inputs, the existing first-layer acceleration
arm already matches `GCode.cpp:10526-10528`: require positive default
acceleration, truncate the resolved value into an unsigned local, require
that local to be positive, then apply `floor(local + 0.5)`. Its surrounding
Rust predicate currently uses `layer_index == 0` rather than the complete
upstream `on_first_layer` predicate. That predicate discrepancy is a separate
candidate, not repaired by this jerk-selection slice.

## Synthetic qualification

Five public synthetic projects use two disconnected 5 mm square boxes, 0.6 mm
high, sliced into three layers on a stock Ender-3-derived Marlin configuration.
Default jerk is 8, ordinary travel jerk is 12, all extrusion-role jerks are 8,
and XY machine jerk limits are 20. Geometry and settings remain identical
except for the initial travel value and the default-zero control.

| Case | Initial travel jerk | Native first-layer travel result | Current Rust result |
|---|---|---|---|
| Relative | `50%` | Eight `M205 X6 Y6` selections | Eight `M205 X12 Y12` selections |
| Absolute | `3` | Eight `M205 X3 Y3` selections | Eight `M205 X12 Y12` selections |
| Disabled initial | `0` | No travel-jerk selection; initial extrusion jerk remains cached | Eight `M205 X12 Y12` selections |
| Ordinary control | `100%` | Travel jerk 12 | Complete ordered output matches |
| Disabled default | `50%`, default jerk `0` | No runtime XY-jerk selections | Complete ordered output matches |

On the second layer the first four cases select ordinary travel jerk 12;
the default-zero case remains disabled. The relative case distinguishes
`travel_jerk` as the reference from the incorrect result `50% × 8 = 4`.

Each project has two fresh Native replays in each of the official and seeded
launch environments. Every within-environment ordered replay comparison
matches after the established single generator-line normalization. These
launch modes share the established producer family; they are not asserted to
be independent ELF confirmations. Both controls also match the current Rust
CLI's complete ordered output in both environments. The three target cases
differ at the selected jerk commands and downstream progress scheduling;
progress lines and all other output remain part of the comparison, unmasked.
No Native-generated output is committed.

## Failure modes and test-first implementation plan

Before production changes, freeze the five synthetic input projects and add
public-API/CLI cases through the existing `native_stream_case::check` seam.
Qualify Native double replay, observe first-layer versus subsequent-layer jerk
commands, and compare the complete ordered API and CLI byte streams with the
existing generator-only comparator. Keep repeatable comparison artifacts.

The cases must detect:

- Ignoring the initial-layer option or resolving percentages over default jerk.
- Treating absolute values as percentages or rounding/truncating jerk.
- Falling back to ordinary travel jerk when the initial value is zero.
- Bypassing the default-jerk positive gate.
- Applying the initial value to later layers or disturbing ordinary controls.
- Accidentally changing writer command order, deduplication, clamping, or
  downstream timing/progress output while fixing selection.

Commit the specification, then the failing behavioral cases, then the minimal
emitter change. Preserve existing adjacent travel acceleration and jerk tests;
run affected cases in both Native environments, `cargo fmt --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, and the WASM core
compile check. Use `cargo nextest run` for Rust execution. Keep every changed
Rust file at or below 400 lines. Completion requires strict whole-stream
API/CLI parity for all five cases, not merely matching extracted jerk values.
