# CLI `--load-settings` partial lists (compose → ProfileSelection)

Upstream: `OrcaSlicer.cpp` `CLI::run` load-settings producer loop
(`:2009-2106`), pin `8500fcdcc`. Rust boundary: `ares-core`
`profiles::load_settings::compose_load_settings` +
`profiles::composition::ProfileSelection` (via
`profiles::entry::with_load_settings_source`), adapter
`ares-cli/src/main.rs`.

## Problem

`compose_load_settings` ends in `ProfileSelection::new(process,
machine, filaments)`, which hard-requires all three names non-empty
("profile selection must include process, machine, and filaments").
Upstream has no such requirement: the `--load-settings` file loop
accepts any subset — machine-only, process-only, or none (and
filament-typed files validate and merge nothing, `:2101-2104`) —
and a missing kind simply contributes no overlay: the embedded
config of that kind is retained. Native accepts and slices
machine-only + witness 3MF (handoff-verified; the E2E reference
proves it at RED time). Worker4's compat17 branch-2 RED (machine
loaded, process embedded) is blocked on this gap.

## Upstream semantics (plain path)

- Machine arm `:2033-2070`: sets `new_printer_name`,
  `printer_settings_id`; records `default_print_profile` into
  `new_default_process_name` (`:2065-2067`) — consumed only by the
  BBL `machine_switch` extruder-count corner (`:3053-3110`), not by
  the plain path.
- Process arm `:2071-2097`: sets `new_process_name`,
  `print_settings_id`, captures `compatible_printers`.
- Missing kind: no overlay; the 3MF's embedded machine/process
  config stays. The `-17` compat gate (`:2575-2606`) adjudicates on
  whichever names ARE loaded (empty names skip that side) — that
  loaded-names semantics is worker4's compat17 slice.
- Duplicate machine/process → `-5` (`duplicate machine config
  file`, `:2034-2040` / `:2073-2079`); unknown type → `-5`
  (`:2099-2104`). Already ported.
- `--load-filaments` without `--load-settings` files: filament
  overlay only (`:2111-2215`); embedded machine/process retained.

## Scope

**Included** (3MF input surface, the existing witness fixtures):

1. `compose_load_settings` accepts partial lists: a missing kind
   contributes no overlay fields and leaves the embedded selection
   for that kind in place. `ProfileSelection` gains the ability to
   express "not selected" per kind (machine/process/filaments),
   replacing the all-three-non-empty assertion for the
   load-settings entry; `catalog.compose` pulls only the loaded
   kinds into the overlay raw.
2. REDs (runtime-native references, existing runner):
   - machine-only + witness `input.3mf` (post_process emptied
     variant): Native slices; Ares today fails the
     `ProfileSelection::new` assertion — RED. GREEN = whole-file
     G-code parity (banner-normalized) with the machine overlay
     applied and the embedded process retained.
   - `--load-filaments` only (no settings files) + the same 3MF:
     same contract for the filament side (embedded machine/process
     retained).
3. Keep duplicate/unknown-type `-5` behaviors unchanged.

**Deferred** (explicit, with owner):

- STL input + partial lists (no embedded base; upstream failure
  surface differs) — follow-up after native probing; owner worker5.
- The BBL `machine_switch` `default_print_profile` completion
  (`:3053-3110`) and `up_config_to_date` re-fetch completion
  (`:2369-2424`) — BBL branches outside the plain path; owner TBD
  (worker1 profile domain when those branches land).
- The `-17` loaded-names adjudication for partially-loaded lists —
  worker4 compat17 (depends on this slice).
- The `inherits` direct-read vs `Catalog::push_import` parent
  resolution gap (worker4 compat17 finding) — separate slice after
  this one, owner worker5 (compose).

## Controls

- No `dbg!`/`println!`, no `#[ignore]`, no `allow`/`expect`, no
  assertion weakening; the RED must compile.
- Native references generated at runtime; only fixtures committed.
- Test inputs (fixtures/argv) change in the RED commit only;
  GREEN touches production code + this spec's record only.
