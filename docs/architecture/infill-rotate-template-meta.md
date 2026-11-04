# ARD: `sparse/solid_infill_rotate_template` meta-language interpreter (scope C)

## Problem statement

`sparse_infill_rotate_template` / `solid_infill_rotate_template` are
`coString` with no constraint (`PrintConfig.cpp:3993-4015`). Ares'
`simple_rotation_angle`
(`group_fills/params/projection/rotation.rs:41-53`) accepts **only** a plain
comma/whitespace-separated list of floats; anything else is rejected as
`UnsupportedProjectFeature`, and the same strings are additionally
pre-rejected in the admission guard (`anchor_projection.rs` sparse
`~:59-64`, solid `~:65-69`). Upstream accepts the full meta-language and
never rejects a template string.

## Upstream evidence (`src/libslic3r/Fill/Fill.cpp`)

`calculate_infill_rotation_angle(object, layer_id, fixed, template)`:

- `Fill.cpp:57-59` — empty template → `deg2rad(fixed_infill_angle)`
  (already ported).
- `Fill.cpp:67` — meta test: `regex_search(template, [+\-%*@'"cm` +
  `search_string])` where `search_string = "/NnZz$LlUuQq~^|#"` (`Fill.cpp:64`).
- `Fill.cpp:206-210` — **non-meta** template: `ConfigOptionFloats::deserialize`
  + `layer_id % size` cyclic list. Matches the current Ares fast path.
- `Fill.cpp:70-77` — tokenize on `[\s,]+`.
- `Fill.cpp:79-83` — interpreter state: `t` cursor, `repeats`, `angle_add`,
  `angle_steps=1`, `angle_start=0`, `limit_fill_z = start_fill_z =
  get_layer(0)->bottom_z()`, per-token `stop[]`, plus `_noop`, `_absolute`,
  `_negative`, `fill_form` (index into `search_string`).
- `Fill.cpp:85-90` — outer loop per layer `i <= layer_id`; the do/while at
  `:96-182` re-reads tokens until the layer's angle is resolved.
- Per token (`Fill.cpp:101-176`):
  - `!` inside token → one-time instruction: `stop[t] = true` (`:106-108`);
  - first char is a digit → `_absolute` (`:110-112`);
  - `strtod` parses `angle_add`; a `%` suffix multiplies it by `3.6`
    (percent of full turn: `angle_add *= 3.6`, `:114-119`);
  - `*n` in token → `repeats = strtol(n)` (`:121-122`);
  - `repeats == 0` → `_noop = true` (`:172-173`);
  - `cs[0] == 'B'` → `angle_steps = default_region_config().bottom_shell_layers`
    (`:125-127`); `'T'` → `top_shell_layers` (`:127-128`);
  - otherwise `fill_form = search_string.find(cs[0])` (`:129-131`); when not
    found (`npos`): `_negative = (cs[0]=='-')`, `angle_steps = abs(strtod)`
    (`:133-134`), and a trailing length-unit converts steps into a Z limit
    (verbatim `Fill.cpp:139-160` — expressions taken as-is, no
    reinterpretation):
    - `'%'` → `limit_fill_z = angle_steps * object->height() * 1e-8`
      (upstream's literal factor — e.g. `50%` ⇒ `50·height·1e-8`),
    - `'#'` → `limit_fill_z = angle_steps * layer_height` (N layer-heights),
    - `'\''` → `limit_fill_z = angle_steps * 12 * 25.4` (feet),
    - `'"'` → `limit_fill_z = angle_steps * 25.4` (inches),
    - `'c'` → `limit_fill_z = angle_steps * 10.` (centimeters),
    - `'m'` → `cs[1]=='m'` ? `angle_steps * 1.` (mm) : `angle_steps * 1000.` (m);
    then `limit_fill_z += fill_z; angle_steps = 0` (`:158-160`).
  - when `angle_steps != 0` remains: `fill_form == npos && !_absolute` →
    `angle_add *= (int)angle_steps`; `idx = i + max(angle_steps-1,0)`; clamp
    `idx` to `layers().size()-1`; `limit_fill_z = get_layer(idx)->print_z +
    sdx * layer_height` (`:163-171`);
  - `repeats = max(repeats-1, 0)` (`:170`); `_absolute` →
    `angle_start = angle_add; angle_add = 0` (`:175-176`);
  - `++t` wraps at `tk.size()`; loop guard `all_of(stop) ? false :
    (t ? _noop : false) || stop[t]` (`:178-182`).
- `Fill.cpp:184-203` — `negvalue` interpolation:
  `(_negative ? limit_fill_z - top_z : top_z - start_fill_z) /
  (limit_fill_z - start_fill_z)`, `top_z = get_layer(i)->print_z`
  (`:185-187`), then the 16-case joint table (`:189-202`):
  | `search_string` idx | char | transform |
  |---|---|---|
  | 0 | `/` | `negvalue` (linear) |
  | 1 | `N` | `-= sin(n*2π)/(2π)` |
  | 2 | `n` | `-= sin(n*2π)/(4π)` |
  | 3 | `Z` | `+= sin(n*2π)/(2π)` |
  | 4 | `z` | `+= sin(n*2π)/(4π)` |
  | 5 | `$` | `= asin(n*2-1)/π + 0.5` |
  | 6 | `L` | `= sin(n*π/2)` |
  | 7 | `l` | `= 1 - cos(n*π/2)` |
  | 8 | `U` | `= 1 - (1-n)^2` |
  | 9 | `u` | `= (1-n)^2` |
  | 10 | `Q` | `= 1 - (1-n)^3` |
  | 11 | `q` | `= (1-n)^3` |
  | 12 | `~` | `= rand()/RAND_MAX` |
  | 13 | `^` | `+= rand()/RAND_MAX - 0.5` |
  | 14 | `|` | `= 0.5` |
  | 15 | `#` | `= _negative ? 0 : 1` |
  then `angle = deg2rad(angle_start + angle_add * negvalue)` (`:201`).
- Data dependencies: `get_layer(i)->{bottom_z, slice_z, print_z}` for
  `i <= layer_id`, `object->height()`, `default_region_config().{bottom,top}_shell_layers`,
  `object->config().layer_height`, `object->layers().size()`.

Upstream **never rejects** an invalid template: non-meta + unparseable
`deserialize` yields an empty `rotate_angles` → `layer_id % 0` UB in the
release binary. Reproducing UB is a non-goal.

## Rust boundary

- `group_fills/params/projection/rotation.rs` keeps the fast path + thin
  dispatch; the interpreter goes in a **separate file**
  `group_fills/params/projection/rotation/meta.rs` (each file ≤400 lines):
  `meta_rotation_angle(template, &ObjectRotationContext) -> Option<f64>`.
- `LayerContext` gains `&'a [PlannedLayer]` (whole `plan.layers`, available
  at `params.rs:52-79`) and `object_height` derived the same way upstream
  reads `object->height()`; `layer_height` comes from `object` config.
- `B`/`T` read `object->print()->default_region_config().{bottom,top}_shell_layers`
  (`:125-128`) — the **print-level process values** (`views.full.process.region`),
  never the current `RegionOptions` (which may carry object/modifier
  overrides).
- `rand()` joints (`~` idx12, `^` idx13): the Native oracle is
  deterministic (no `srand` on the slicing path → glibc default-seeded
  TYPE_3 stream, first draws `1804289383, 846930886, 1681692777`).
  `GlibcRandom` reproduces that sequence and lives in `thread_local`
  state; `slice_source` calls `reset_rand_state()` at the start of each
  slice to give fresh-process equivalence — without the reset a second
  slice on the same worker thread (WASM worker / API embedder) would
  continue the stream and emit different G-code than the oracle.
- Admission guards (`anchor_projection.rs` sparse/solid template checks):
  remove the `is_none()` rejections **only after** the interpreter lands.
  `simple_rotation_angle`'s residual `None` (unparseable non-meta string)
  maps to the upstream UB domain.

## Documented upstream-UB deviation

Ares feeds every non-meta token to `strtod`-prefix parsing, so every
string produces a numeric list and the upstream `layer_id % 0` UB domain
(`deserialize` → empty `rotate_angles`) is unreachable in practice; the
theoretical empty-token case returns angle `0.0` instead of dividing by
zero — a deliberate deviation: matching UB/crash is not a parity goal.

## Documented rand() deviation

`GlibcRandom` is **per-worker-thread**, upstream's `rand()` is
process-global. Sequential slicing (the only mode Ares runs, and the
mode the Native oracle exercises) is bit-identical; if Ares ever
parallelized layer projection, draws would partition across threads
differently than upstream's single interleaved stream. `reset_rand_state()`
per `slice_source` guarantees per-slice determinism even when the
embedding process slices repeatedly.

## Included / deferred

- Included: meta-language for both `sparse_` and `solid_` channels;
  `_absolute`/`_negative`/`_noop`/`stop`/`repeats` semantics; `B`/`T`;
  all length units (`%`, `#`, `'`, `"`, `c`, `m`, `mm`); all deterministic
  joints (`/ N n Z z $ L l U u Q q | #`); the `*3.6` percent-of-turn form.
- Deferred: unparseable non-meta UB domain (deviation above);
  `infill_direction`/`align_infill_direction_to_model` (already ported).
  `~`/`^` are included — RED measured deterministic draws and the glibc
  default-seeded stream is reproduced exactly.

## RED matrix (condition ②)

`orca_parity/infill_rotate_template.rs`, Native official+seeded, fresh
roots, `tests/parity/overhang.stl`, `fill_multiline=1`:

| case | channel | template | branch coverage |
|---|---|---|---|
| T0 | sparse | `+45` | incremental steps |
| T1 | sparse | `45` | absolute (leading digit) |
| T2 | sparse | `+10%` | percent-of-turn (`*3.6`) |
| T3 | sparse | `+5*3` | `*n` repeats |
| T4 | sparse | `0,45!` | `!` one-time |
| T5 | sparse | `+30B` | `B` bottom_shell_layers steps |
| T6 | sparse | `+30T` | `T` top_shell_layers steps |
| T7 | sparse | `+45#2` | `#` = 2 layer-heights |
| T8 | sparse | `+45'1` | `'` feet |
| T9 | sparse | `+45"1` | `"` inches |
| T10 | sparse | `+45c1` | `c` centimeters |
| T11 | sparse | `+45m5` | `m` meters |
| T12 | sparse | `+45mm5` | `mm` millimeters |
| T13 | sparse | `+45%50` | `%` percent-of-height |
| T14 | sparse | `+90N4` | `N` sinus joint |
| T15 | sparse | `+90Z4` | `Z` sinus joint |
| T16 | sparse | `+90$4` | `$` arcsin joint |
| T17 | sparse | `+90L4` | `L` quarter-circle joint |
| T18 | sparse | `+90U4` | `U` squared joint |
| T19 | sparse | `+90Q4` | `Q` cubic joint |
| T20 | sparse | `+90|4` | `|` middle joint |
| T21 | sparse | `+90#4` | `#` end joint |
| T22 | solid | `+30` | solid channel meta |
| T23 | solid | `20,70` | solid channel non-meta list (already-ported fast path, regression control) |
| T24 | sparse | `0,45,90` | sparse non-meta list (regression control) |
| T25 | sparse | `+30~2` | `~` rand joint — determinism probe (3× replay compare) |
| T26 | sparse | `+30^2` | `^` rand joint — determinism probe |

Each case writes `witness.json` (angle-relevant G-code + status); T25/T26
additionally diff 3 fresh Native runs to classify deterministic vs UB.

## Non-goals

- No GUI-side option changes; `PrintConfig` domain already boundless.
- No `sparse_infill_pattern` guard changes; rotation applies only to
  already-supported patterns.
