# procgcode-dynamics oracle on raw OptionsZCorrector semantics

Status: Proposed. Oracle-side fix only; no Ares production code and no test
assertion changes.

## Problem

`runner::adjacent_motion::marlin2_option_moves` failed in full parity runs
(official and seeded) at `adjacent_motion.rs:198` ("vertex identities") once
the OptionsZ moves chain (`13f628158`) landed in the stack.

`adjacent_motion.rs:172` reads its Native vertex identities from
`processor_dynamics_reference`, i.e. `ARES_PROCGCODE_DYNAMICS_BIN`. That oracle
(`tools/chain-probe/procgcode-dynamics.nix`) is built as
`(import ./procgcode-stable-options.nix).overrideAttrs`, so it carries
`procgcode-stable-options.patch`: `OptionsZCorrector::remap_after_time_calculation`
re-targets the pending `m_move_id` after `calculate_time` inserts actual-speed
vertices. That is the corrected, diagnostic-only behavior.

Ares targets **raw** Native, stale index included. The
observed diff is exactly that difference and nothing else: 352
identity differences in two runs, one per marker. For the pause marker
(gcode 381) the corrected oracle moves the marker to index 550, while raw
semantics move the stale-index element `192/10/1` there. The custom marker
(gcode 500) shows the same pattern with `433/10/0` at index 744.

## Upstream boundary

- `OptionsZCorrector::set` / `update`, `GCodeProcessor.hpp:688-705`.
  `set()` stores `moves.size() - 1`, and `update()` copies and erases at that
  stored index without any remapping.
- The marker tags store the vertex and then call `set()`: `GCodeProcessor.cpp:3292-3296`
  (color change), `:3305-3309` (pause, followed by `process_custom_gcode_time`
  → `simulate_st_synchronize`, `:5917-5921`, `:6011-6014`) and `:3315-3318`
  (custom). The next extrusion calls `update()` at `:3929`. Between the two,
  any `calculate_time` flush can insert vertices; the periodic refresh at
  `:4187-4188` is one example.
- `GCodeProcessor::calculate_time`, `:5951-6009`. It inserts actual-speed
  vertices into `result.moves` (`:5962-5997`) and renumbers only the time
  blocks (`:6003-6008`). The stored `m_move_id` is never adjusted, and that is
  the raw behavior the oracle must reproduce.

## Change

1. `tools/chain-probe/procgcode-dynamics.nix`: import `./procgcode.nix` (raw)
   instead of `./procgcode-stable-options.nix`, and update the header comment.
   `procgcode-dynamics.patch` is unchanged. Its hunks (`OrcaSlicer.cpp`
   `--dump-move-dynamics`, `GCodeProcessor.cpp` `:2445` travel gate and `:2520`
   PA init) don't overlap the stable-options hunks (`hpp:690`, `cpp:6006`).
   This gets checked with `git apply --check` on a clean 8500fcdcc tree.
2. `crates/ares-cli/tests/orca_parity/runner/cache/processor_diagnostic.rs`:
   `Recipe::Dynamics::patches()` drops `procgcode-stable-options.patch` so the
   cache identity describes the oracle that was actually built. This is a
   harness identity change, not an assertion change.
3. Docs: `tests/parity/motion-attributes/README.md` and
   `docs/architecture/web-motion-attributes.md` no longer list stable-options
   as part of the dynamics oracle.
4. CI: `scripts/ci/build-oracle.sh` builds the `procgcode-dynamics` variant by
   evaluating `tools/chain-probe/procgcode-dynamics.nix`. The derivation
   therefore changes with item 1, and the `native-oracle-build` matrix job
   rebuilds it. No `tier1.yml` edit is needed, and the `ARES_PROCGCODE_DYNAMICS_BIN`
   wiring at `tier1.yml:152/269/374` stays the same.
5. The canonical control set is updated to the new store path.

## Deferred / unchanged

- `procgcode-stable-options` is retired:
  beta upstreamed the correction (`GCodeProcessor.hpp:1105-1116`), the patch
  no longer applies to the beta tree, and its former users compare against
  raw Native beta.
- `merged-oracle` (the marker-time binaries) keeps stable-options. It doesn't
  feed `ARES_PROCGCODE_DYNAMICS_BIN`, so changing it is a separate decision.
- Stealth mode (ARD-0052).

## Evidence plan

1. **Verification:** run the raw procgcode `--dump-moves` on
   the oracle input G-code and compare `(gcode_id, type,
   internal_only)` with the Ares identities. Expected: equal.
   Also run the current dynamics binary, which should differ at exactly the
   352 indices.
2. **New derivation:** build it, record the store path and sha256 of
   `.orca-slicer-wrapped`, and diff its `--dump-move-dynamics` identities
   against raw `--dump-moves` on the same input. Expected: equal.
3. **Fail on base** (pre-GREEN Ares) with the new oracle:
   `marlin2_option_moves` fails with the `:198` vertex-identities assertion.
4. **Pass at head:** with the new oracle, `-E 'test(/runner::adjacent_motion::/)'`
   passes, and so do `gcode_moves_oracle`, `runner::adjacent_color`, fmt,
   strict clippy and wasm32 clippy.
