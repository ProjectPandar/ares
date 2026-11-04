# Web Feature Type legend: per-role and travel estimated times

Rewrite slice of `libvgcode` `ExtrusionRoles`
and the `ViewerImpl::load` travel-time accumulator, consumed by the upstream
Feature Type legend Time/% columns.

## Upstream boundary

| Upstream | What it does | Ares destination |
|---|---|---|
| `libvgcode/src/ExtrusionRoles.hpp:15-31`, `ExtrusionRoles.cpp:9-37` | `std::map<EGCodeExtrusionRole, Item{times[TIME_MODES_COUNT]}>`: `add` sums vertex times per role; `get_roles` returns present roles in key (enum) order; `get_time(role, mode)` is 0 for absent roles | new `crates/ares-vgcode/src/extrusion_roles.rs` (`ExtrusionRoles`) |
| `ViewerImpl.cpp:1007-1018` (`load`) | per vertex: `m_travels_time[j] += v.times[j]` when `v.type == Travel`; `m_extrusion_roles.add(v.role, v.times)` when `v.type == Extrude` | `Viewer::load` in `crates/ares-vgcode/src/viewer.rs` (existing loop at :73-92; file is 342 LOC, so the role/travel state lives in `extrusion_roles.rs`) |
| `ViewerImpl.cpp:871,875` (`reset`) | clears both | `Viewer::load` re-initialises them |
| `ViewerImpl.hpp:140-147`, `Viewer.cpp:325-333` | `get_extrusion_roles`, `get_extrusion_role_estimated_time(role)` and `get_travels_estimated_time()` in `m_settings.time_mode` | `Viewer` methods of the same names |
| `GCodeViewer.cpp:3151` | `total_estimated_time = time_mode.time > 0 ? time_mode.time : m_viewer.get_estimated_time()` | web legend (`time` from the existing `Estimation` transfer, viewer total otherwise) |
| `GCodeViewer.cpp:3410-3418` | `role_time_and_percent`, `travel_time_and_percent`: `time / total` or 0 | web legend |
| `GCodeViewer.cpp:3631-3672` (FeatureType case) | rows from `get_extrusion_roles()`; Time cell `time > 0 ? short_time(get_time_dhms(time)) : ""`; % cell `"0"` when percent == 0, `"%.1f"` of `percent*100` when `> 0.001`, else `"<0.1"`; Travel row gets the same Time/% cells | `web/app/src/preview/Legend.tsx` Feature Type rows (existing `shortTime`, `timeDhms`, `printf` in `toolPositionFormat.ts`) |

`Bitset`, `CogMarker`, `ToolMarker` are not in scope (the latter two are compiled
out upstream: `Types.hpp:8` `VGCODE_ENABLE_COG_AND_TOOL_MARKERS 0`).

## Included

1. `ExtrusionRoles` port. Storage is a fixed `[Option<[f32; TimeMode::COUNT]>; GCodeExtrusionRole::COUNT]`
   indexed by role: same iteration order as the `std::map` (role keys are the enum
   order, `Types.hpp`), same f32 `+=` order (vertex order), no allocation. `get_roles`
   yields present roles; `get_time` returns 0.0 for absent ones.
2. `Viewer::load` accumulates `travels_time` and `extrusion_roles` in the same loop
   that already sums `total_time`, matching `ViewerImpl.cpp:1007-1018` statement order.
3. `ares-wasm`: the toolpaths transfer carries, for the Normal time mode (the web's
   only mode, as in `estimation.rs`), the present role ids plus their f32 times and
   the travel f32 time. Small typed arrays, no JSON for the per-role data.
4. Web: the Feature Type legend's role list comes from the transferred role ids
   (replacing `usedRoles` in `web/app/src/viewport/roles.ts`, which recomputes the
   same set in JS; it is deleted when unused). Each role row and the Travel option
   row gain Time and % cells formatted exactly as `GCodeViewer.cpp:3644-3669`,
   with headers `Line Type`, `Time`, `%` (`:3679-3681`; Usage deferred). Visibility toggles stay as
   they are.

## Deferred (adjacent upstream behaviour, separate slices)

- Usage columns: `used_filament_per_role` (`GCodeViewer.cpp:3652-3656`, from
  `PrintStatistics::used_filaments_per_role`, already transferred by
  `takePrintStatistics` but not consumed) and the Travel row distance/move count
  (`:3664-3668`, `total_travel_distance`, `total_travel_moves`).
- Silent time mode and the time-mode switch (`m_settings.time_mode` stays Normal).
- Column offset/alignment math (`calculate_offsets`, `:3681`); the web uses a table.
- LayerTime and ColorPrint estimated-time display (`:3152-3154`).

## Existing Ares scaffolding

`usedRoles` (JS) is a temporary compatibility shell for `ExtrusionRoles::get_roles`
and is replaced, not wrapped. The `Viewer::load` total-time loop is the upstream loop;
it is extended, not duplicated.

## Oracle and tests (TDD, E2E-first)

- Native oracle: `tools/chain-probe/role-times-oracle.hpp` + `role-times.nix`, same
  shape as `custom-gcode-overview-oracle.hpp`: real `LibVGCodeWrapper` conversion of
  the procgcode `GCodeProcessorResult`, the verbatim `ExtrusionRoles` (`ExtrusionRoles.cpp`
  compiled as is) and the `ViewerImpl.cpp:1007-1018` loop, plus the extracted
  FeatureType cell formatting (`GCodeViewer.cpp:3410-3418, 3640-3669`, `_u8L` identity).
  Output per fixture: role ids, f32 time bits per role, travel f32 bits, total,
  and the rendered Time/% cell strings.
- RED 1 (`crates/ares-vgcode/tests/extrusion_roles.rs`, Native-compared): the
  fixture G-codes already under `tests/parity/` (motion-attributes, marker-time,
  color-change normal) converted through `ares-vgcode`; role list, per-role and
  travel times compared bit-exact to the oracle JSON. Fails before the port
  (missing API → compile is not a RED; the RED commit adds the API with `todo!`-free
  zero-returning bodies so the failure is an assertion on values).
- RED 2 (web e2e, in `toolpathPreview.ts` Feature Type check): legend rows' Time
  and % text equal the oracle cells for the preset-slice plate (oracle run in-test
  like `colorRanges.ts`). Fails on the missing cells.
- GREEN: port + wasm transfer + legend cells. Gates: directed nextest
  (`extrusion_roles`, existing `viewer` tests), clippy strict, fmt, wasm32 build,
  web build/lint/typecheck, gzip < 125000, browser desktop `presetSlice`/`slice`.

## Failure modes enumerated before coding

1. Roles summed in f64 or in a different order → last-bit drift vs Native (bit compare).
2. Roles listed by first appearance instead of enum order → row order mismatch.
3. Non-Extrude vertices (Wipe, Retract, Seam options) counted into a role.
4. Travel time including Wipe moves (upstream: `EMoveType::Travel` only).
5. Total taken from the viewer when `PrintEstimatedStatistics` Normal time is > 0, or vice versa.
6. `"<0.1"` vs `"0"` boundary: percent exactly 0 vs `<= 0.001`.
7. Empty Time cell when `time == 0` rendered as `0s`.
8. wasm transfer taking the wrong time mode or losing f32 bits through JSON.
