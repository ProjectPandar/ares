# Spec: admit `infill_anchor_max < 0.05` as upstream `dont_connect`

Source-cited rewrite slice.

## 1. Problem statement

`crates/ares-core/src/project_slice/prepare_infill/bridge_over_infill/transaction/anchor_projection.rs:103-104`
rejects any projected `infill_anchor_max < 0.05` (absolute mm after percent
resolution) with `UnsupportedProjectFeature("infill_anchor_max")`. Upstream
OrcaSlicer treats the same condition as a supported mode, not an error:
`FillParams::dont_connect()` (`src/libslic3r/Fill/FillBase.hpp:52`) is exactly
`anchor_length_max < 0.05f`, and both connect dispatch sites honor it:

| Upstream site | Behavior when `dont_connect()` |
| --- | --- |
| `FillRectilinear.cpp:3366-3373` (rectilinear family `fill_surface`) | `chain_polylines` + `append`; `connect_infill` skipped |
| `FillBase.cpp:1820-1828` (`chain_or_connect_infill`, used by non-rectilinear sparse patterns via `Fill.cpp` dispatch) | `chain_polylines` + `append`; `connect_infill` skipped |
| `Fill.cpp:978-989` (project params) | `anchor_length_max` / `anchor_length` resolved (percent→spacing), `anchor_length = min(anchor_length, anchor_length_max)` — no rejection |
| `FillBase.cpp:1584-1585,2251-2252` (`connect_infill` asserts) | `anchor_length_max >= 0.01` asserted **inside** `connect_infill` only — unreachable on the `dont_connect` path, so upstream genuinely supports `infill_anchor_max = 0` |

Ares already carries the equivalent plumbing: `fill/plane_path.rs:146`
(`anchor_length_max < 0.05 || density > 0.5` → chain instead of
`connect_infill`), `fill/rectilinear/surface.rs:153` (`connect_contours(..,
anchor_length_max < 0.05, ..)`), `fill/rectilinear/links.rs` `dont_connect`
flag, and `sparse_anchoring/projected.rs:11-25` mirrors the
`Fill.cpp:978-989` projection including the `min(anchor, anchor_max)` clamp.
The `anchor_projection.rs:103` guard is therefore an admission-time false
negative: it rejects a value domain the ported pipeline already implements.

## 2. Rust destination boundary

- **Removed**: the `anchor_max < 0.05` conjunct at
  `bridge_over_infill/transaction/anchor_projection.rs:103-104`. The
  `!anchor_max.is_finite()` conjunct stays, as does `anchor < 0.0` — both
  are unreachable through sanctioned option parsing upstream:
  `infill_anchor`/`infill_anchor_max` are `coFloatOrPercent`
  (`PrintConfig.cpp:3137-3160` / `:3165-3185`) with `sidetext "mm or %"`,
  `max_literal = 1000`, and **no `min`/`min_literal` bound**; `0` is an
  explicit enum value (`"0 (no open anchors)"` / `"0 (Simple connect)"`).
  Negative or non-finite values never reach slicing — these guards remain a
  defensive no-op, unchanged by this slice.
- **No production changes elsewhere** are expected: `dont_connect` behavior is
  already ported in the fill backends listed above. If GREEN exposes a gap in
  a rectilinear-family backend, it is reported, not patched silently.
- The `anchor < 0.0` guard on `infill_anchor` (anchor_projection.rs:106-107) is
  unrelated to this slice and unchanged.

## 3. Included / deferred behavior

**Included**: `infill_anchor_max` absolute `< 0.05` (e.g. `0`) and percent
forms resolving `< 0.05` on sparse (non-solid) infill; whole-output parity in
official + standard seeded modes; `connect_infill` must be absent from the
emitted path for these inputs (asserted indirectly by whole-output equality —
upstream emits chained polylines only).

**Deferred**: Stars/multiline infill interactions are explicitly excluded —
multiline/Stars is a separate slice; this slice fixes
`fill_multiline` = 1 and uses only the rectilinear family dispatch at
`FillRectilinear.cpp:3366`. Solid/bridging surfaces force
`anchor_length(_max) = 1000` upstream (`Fill.cpp:975-977`), so
`dont_connect` never applies to them — no surface-type behavior changes here.
TPMS/3D-honeycomb `anchor_length_max` parameters are structurally separate
(`FillTpms*.cpp` anchor fields) and unchanged.

## 4. Evidence plan (RED → GREEN)

Test location: `crates/ares-cli/tests/orca_parity/infill_anchor_dont_connect.rs`
(new module under `orca_parity`), reusing the existing `overhang.stl` fixture
(`tests/parity/overhang.stl`, SHA256
`a7f79fdf8a5c136095b6ef2384de24dcaa3166c7df4e92768a434c454db55031`) and the
standard Creality Ender-3 preset plumbing (`adaptive_pressure_advance.rs`
`settings()` pattern — classic walls, no tower, comments on).

Contexts (each required in **official** and **standard seeded** modes, fresh
native cache + artifact root per run):

| Case | sparse_infill_pattern | sparse_infill_density | infill_anchor_max | infill_anchor | Purpose |
| --- | --- | --- | --- | --- | --- |
| C0 | `rectilinear` | `20%` | `0` | `0` | Absolute-zero dont_connect via `FillRectilinear.cpp:3366` chain path |
| C1 | `zigzag` | `20%` | `0` | `0` | Same family, second pattern coverage of the 3366 dispatch |
| C2 | `rectilinear` | `20%` | `0.02` | `0` | Below-threshold nonzero absolute: dont_connect still selected (<0.05) |

All contexts fix `fill_multiline = 1`, default seam/tower settings (no prime
tower), single extruder, and leave `sparse_infill_line_width` at preset default.
Expected today: each fails at `UnsupportedProjectFeature("infill_anchor_max")`
— the RED witness is that Orca slices these inputs and Ares rejects them.
GREEN deletes the `anchor_max < 0.05` conjunct only; no legacy fallback, no
assertion weakening, no comparator change.

## 5. Non-goals

- No change to `connect_infill` semantics for `anchor_length_max >= 0.05`.
- No admission/ordering work in the multiline/Stars scope.
- No `unsupported()` plumbing refactor; other guards untouched.
