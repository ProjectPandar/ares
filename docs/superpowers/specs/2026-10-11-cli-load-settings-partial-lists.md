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
machine-only + witness 3MF (the E2E reference proves it at RED
time). The compat17 branch-2 RED (machine
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
  loaded-names semantics is the compat17 slice.
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

**Deferred** (explicit):

- STL input + partial lists (no embedded base; upstream failure
  surface differs) — follow-up after native probing.
- The BBL `machine_switch` `default_print_profile` completion
  (`:3053-3110`) and `up_config_to_date` re-fetch completion
  (`:2369-2424`) — BBL branches outside the plain path (profile domain
  when those branches land).
- The `-17` loaded-names adjudication for partially-loaded lists —
  compat17 slice (depends on this slice).
- The `inherits` direct-read vs `Catalog::push_import` parent
  resolution gap (compat17 finding) — separate compose slice after
  this one.

## Addendum: machine-switch merge semantics (2026-10-11)

The machine-only RED exposed three output-side semantics of the
upstream `machine_switch` path; all three belong to this slice,
while the `-17` rejection adjudication stays
in `project_slice/compatibility.rs`:

- **`(auto)` process rename** (`OrcaSlicer.cpp:2660-2702`, gated
  `is_bbl_3mf && machine_switch`): the retained embedded process
  name gains the `"(auto)"` suffix in the merged
  `print_settings_id` (covered slots). Computed on
  `project.documents.bbl_family_3mf`.
- **Compatible-printer append** (`:3050-3051`, `machine_switch`
  without the bbl gate): the merged `print_compatible_printers`
  keeps the embedded list and pushes the loaded machine's system
  name (upstream pushes unconditionally).
- **`different_settings_to_system` marker** (`:3056-3070`): slot 0
  gains the `compatible_printers` key (upstream dedup) before the
  existing prime-tower marker append (`:3688-3698`), producing
  `compatible_printers;enable_prime_tower` in the single slot.

`machine_switch` (merge-side port of `:2622-2640,2585-2596`)
requires: a loaded machine, no loaded process, an embedded printer
  name differing from the loaded one, and the loaded name absent
  from the embedded `print_compatible_printers`. GREEN = the two
  REDs converge whole-file under these semantics; the compatible
  (machine+process) and STL surfaces stay byte-identical
  (no `machine_switch` on those paths).

## Addendum: review-confirmation notes (2026-10-11)

1. **Process-only control**: the composition accepted a process-only
   list from the start of the GREEN
   (`orca_parity_load_settings_3mf_process_only_whole_file`, fixture
   `prusa-process.json` whose `compatible_printers` names the embedded
   machine, branch 3 `:2597-2606`); it converged whole-file without a
   production change, so no RED phase exists for it — recorded here
   as an always-green control.
2. **Unselected typed defaults never overwrite embedded values**:
   `unselected_profile` fills the not-loaded kind's typed options with
   registry defaults only to satisfy the `ProjectSettings` type. In
   the 3MF composite path those typed options are never read for the
   overlay: `merged_profile_raw` consumes the composed profile's
   `settings.project.preset` fields only (identity names, filament
   counts, `print_compatible_printers`), and every option value flows
   from the RAW view — `profile_raw` carries only the loaded files'
   keys (`Catalog::merge` extends `raw` per merged record), applied
   per-key onto the embedded base exactly like upstream
   `update_full_config` (`OrcaSlicer.cpp:2750-2812`), where an empty
   `load_machine_config`/`load_process_config` simply contributes no
   keys. The typed rebuild (bed classification, legacy handling)
   reads the merged raw, not the composed typed options — the same
   separation as the from:system composition.
3. `unselected_profile` takes a closed `UnselectedKind` (machine /
   process) — filament lists are never partial in this entry, so no
   panic branch exists (type-level exclusion).

## Controls

- No `dbg!`/`println!`, no `#[ignore]`, no `allow`/`expect`, no
  assertion weakening; the RED must compile.
- Native references generated at runtime; only fixtures committed.
- Test inputs (fixtures/argv) change in the RED commit only;
  GREEN touches production code + this spec's record only.
