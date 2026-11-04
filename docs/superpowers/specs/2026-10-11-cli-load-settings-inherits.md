# CLI `--load-settings` user presets: direct field load, no inherits resolution

Upstream: `OrcaSlicer.cpp:1953-2010` `load_config_file`, pin `8500fcdcc`.
Rust boundary: `ares-core` `profiles::catalog` import path
(`push_import` → `insert` → `merge`/`collect_positions`) consumed by
`profiles::load_settings::compose_load_settings`; closes the
"--load-settings inherits gap" found by the compat17 slice, whose
fixtures were flattened to avoid it.

## Problem

Ares resolves imported presets' `inherits` chains at compose time:
`Catalog::merge` walks `collect_positions`, linking an imported
preset to any same-kind record whose name matches — whether the
parent file is another import in the same list or (in principle) a
vendor preset — and a resolvable-but-absent parent fails with
`parent profile '{parent}' was not found for '{name}'`. Upstream CLI
**never resolves `inherits` on this path**: `load_config_file` loads
each file's fields directly into its own `DynamicPrintConfig`
(`config.load_from_json`, `:1967`), `inherits` is only read as a
value — the machine/process arms capture it as the system name for
`from: user` files (`:2038-2041,2084-2087`) — and the overlay stage
(`:2750-2812`) applies exactly the loaded fields. There is no
inherits recursion at all on this path: the `inherits` reads in
`load_config_file` are commented out upstream (`:1987`, `:1991`),
so not even a parse failure can route down the chain. A parent file that
is also listed never contributes its fields to the child: machine and
process lists take one file per kind (duplicates reject `-5`,
`:2034-2040,2073-2079`), and multiple `--load-filaments` slots each
overlay their own file per slot (`:2111-2215`).

## Catalog scope on this path

`compose_load_settings` starts from `Catalog::empty()` and imports
only the listed files — vendor presets are never present, so today's
chain walk can only link imports to each other. The change removes
chain resolution for the import path entirely: every import composes
as its own single-record chain (its fields, `load_config_file`
direct-load semantics). The vendor path (`parse_preset_bundle` →
`push_vendor`) keeps full `PresetBundle` chain semantics untouched;
the `missing_parent` skip machinery stays vendor-only.

## Scope

**Included** (one boundary change + two REDs):

1. The load-settings/load-filaments import path performs **no
   `inherits` chain resolution whatsoever**: each file's fields load
   directly (`:1953-2010`), `inherits` is consumed as a value only —
   the system name for `from: user` files wherever the merge consumes
   it (the partial-list `machine_switch` append; the `-17` gate's
   name plumbing stays in the `compatibility.rs` interface).
2. RED A `orca_parity_load_settings_3mf_user_inherits_whole_file`:
   fixture `user-machine.json` = witness `machine.json` with
   `"from": "user"` and
   `"inherits": "Creality Ender-3 0.4 nozzle"` (its own system
   identity, so system-name semantics match the from:system witness
   and the change isolates the load boundary), argv mirrors the
   compatible E2E (settings `user-machine.json;process.json`,
   `compatible-filament-0.json`, arrange 0). Native loads and
   slices; Ares today fails parent resolution → RED. GREEN =
   whole-file parity (banner-normalized). The native reference also
   adjudicates whether the merged config echoes the `inherits` key
   in the full-config dump (upstream `:2004` erase is commented
   out); if it echoes, GREEN admits it for user imports
   (`catalog::raw::applies` currently filters `inherits`).
3. RED B (no-chain control, absent parent): `--load-filaments
   child-filament.json` where the child declares
   `"inherits": "Inherits Parent Filament"` (not listed) and omits
   `filament_flow_ratio`. Native direct-loads the single slot
   (`:1987,1991` chain reads commented out) and slices; Ares today
   chain-resolves and fails with a missing-parent error → RED.
   GREEN = direct overlay, parent never consulted.
   The originally planned same-list pair control
   (`child;parent`) is unreachable as a whole-file witness: the
   reference Native 2.4.2 deterministically SIGSEGVs on ANY
   two-entry `--load-filaments` list in this environment (verified
   with unrelated pairs, both orders, twice each);
   `parent-filament.json` is kept as the documented counterpart of
   that investigation.
4. Duplicate-machine/process and unknown-type `-5` behaviors are
   unchanged.

**Deferred** (explicit):

- User-preset process files with absent parents — same boundary
  change covers them mechanically; an E2E control waits for its own
  fixture if the matrix needs one (follow-up).
- `inherits`-key echo for `from: system` files and the
  `different_settings_to_system`/`inherits_group` bookkeeping for
  user imports beyond what the RED references show.
- GUI `PresetBundle` import semantics (`PresetBundle.cpp:1521-1525`
  skip-on-missing-parent) — outside the CLI slice family.

## Controls

- No `dbg!`/`println!`, no `#[ignore]`, no `allow`/`expect`, no
  assertion weakening; the RED must compile.
- Native references generated at runtime; only fixtures committed.
- Test inputs (fixtures/argv) change in the RED commit only;
  GREEN touches production code + this spec's record only.
