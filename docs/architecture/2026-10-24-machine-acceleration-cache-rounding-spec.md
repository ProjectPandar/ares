# SPEC: machine print-acceleration cache ties-to-even rounding

## Upstream boundary and architectural constraints

`OrcaSlicer/src/libslic3r/GCodeWriter.cpp:38-41::apply_print_config` converts the
machine extruding acceleration limit with `std::lrint`; Klipper additionally
converts the X and Y limits with `std::lrint`. Under the normal rounding mode,
these operations round to nearest with ties to even. The flavor gate is at
lines 32-35; positive Klipper axis limits participate in the minimum at 42-43.
`set_acceleration_internal`, lines 209-219, applies the resulting print cache
before command suppression and serialization.

The Rust destination is
`gcode_emit/motion/options/helpers.rs::machine_acceleration_limit`. Its three
cache conversions currently call `rounded_acceleration`, which uses
`(value + 0.5).floor()`. This differs at even half-integers. The options assembly
is a temporary compatibility shell for the cited upstream writer cache, not an
independently designed acceleration pipeline.

Change only those three conversions to `round_ties_even`,
keeping their current u32 cache type, flavor admission, positive-limit filtering
and minimum. Do not create a shared lrint helper, refactor the existing jerk
conversion from `0bb7523bf`, or change the generic `rounded_acceleration` helper.
The travel cache uses upstream `std::round` and must remain unchanged. Preserve
process/object rounding, raw machine-envelope output, cache suppression and
emitter control flow.

Deferred: processor limits and time estimates, auto_lift/M73, the tokenizer,
fan and zero-E, and value.rs/coInts. No changes to those files, masks,
processor compensation, new abstractions or unrelated numeric validation.
The three-cache source boundary includes Klipper X/Y conversion, but this RED
uses the common extruding-cache witness, not additional axis cases.

## Saved project and Native witness

Start from the existing GUI-generated Ender3 project
`tests/parity/orca_cli_ender3.3mf`, with the prime tower disabled. All three cases
set `default_acceleration=1000` and `initial_layer_acceleration=500`, which
activate body acceleration commands. Without these requests, the original
fixture does not exercise the writer cache. Other process settings, geometry,
templates, jerk, temperature, pressure advance and fans remain unchanged.

The only per-case difference is `machine_max_acceleration_extruding`:

| Case | Value | SHA256 |
|---|---|---|
| Stock control | Stock | b1a3ee564b1e78d1b87f6957f2177f6cd528906ce4790218a0dca410b4191bb8 |
| Even-half target | 400.5 | 110b8b97b5faabd9365e3dae2d1fb9f15339dff4337da95f96337c8ec8952977 |
| Odd-half control | 401.5 | 0eca982752c861b52a980dd46dd0fc4514748780705fe30a99cc1c9f3d9c595b |

The target's first normalized difference is byte 1328: Native emits
`M204 S400`, whereas Ares emits `M204 S401`. The odd-half control rounds to 402
in both implementations. Both controls match over their complete streams.
These valid saved float options deliberately distinguish rounding semantics;
there is no claim about GUI decimal precision or recommended physical tuning.

The target also moves later M73 records because the emitted acceleration changes
motion timing. Retain those differences in the full comparator. They are not a
request to edit time-estimator code. If a timing discrepancy remains after the
cache correction, report it rather than expanding this slice or hiding it.
The initial raw machine-envelope command is not the writer's rounded body cache
and remains unchanged.

Six fresh Native slices across the three active cases replay identically; all
Native and CLI exits are zero. Earlier inactive-cache nulls and a Native-rejected
MakerWare probe are not added to this test slice.

## RED seam and failure modes, written before implementation

Freeze the three archives. Use direct calls to the existing
`native_stream_case` public Native/API/CLI seam. Fresh Native replay must qualify
first. A Native callback reads the saved cap and initial-layer request from the
input and checks the first body M204 S field against the upstream ties-even
clamp. This is a source-derived observation, not a filter or hardcoded Native
coordinate/count golden. It must distinguish the body cache from the initial
raw machine-envelope command.

Compare complete Native/API/CLI output in official and seeded modes, retaining
M73, headers, configuration and footers with only the existing generator-line
normalization. Require raw API/CLI equality. Commit no Native reference outputs.
Expected RED: the target fails at the writer command; both controls pass; all
fresh Native replays and input-derived callbacks qualify. Commit RED before
production changes.

Failure modes: even-half versus odd-half conversion; silently inactive body
cache; changing generic process or travel rounding; confusing raw envelope data
with the writer cache; changing flavor/minimum admission; and masking M73
movement. The controls and Native callbacks establish reachability and source
semantics independently of the Ares implementation.

GREEN: the minimal three-conversion change only, with RED tests and
inputs unchanged; fmt,
workspace all-target clippy, and the same three cases in official then seeded
mode. Preserve normal group registration, root registries and the 400-line Rust
file limit.
