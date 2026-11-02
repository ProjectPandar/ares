# Addendum: explicit project-preset bytes in adapters

Status: accepted with runtime-generation, uniform-supply and inventory-delta conditions. Companion to `project-derived-preset-refresh.md`; the approved core contract is unchanged.

## Source boundary

Pinned OrcaSlicer revision: `345efec64c7b95a05ce5cf96df0dde66a158835c`.

`src/OrcaSlicer.cpp:2112–2117` lazily creates the system preset resolver. The project's own preset-loading calls at `:2788–2816` (filaments) and `:3163–3200` (printer/process) invoke `PresetBundle::resolve_system_preset`. `src/libslic3r/PresetBundle.cpp:724–758` finds the owning vendor and reads the installed `data_dir/system` vendor manifest/cache, otherwise the shipped `resources/profiles` tree. `load_source_vendor` (`:653–695`) loads the base filament library and the vendor's defaults/inheritance through the canonical vendor loader.

This addendum ports the adapter's explicit input-supply role, not Native's host search policy. Core still owns `Preset::load_external_config`, `project_different_keys`, and the CLI refresh/application rules described in the companion ARD. `GCode::append_full_config` remains serialize-only.

## CLI contract

Add exactly these caller-facing options to `ares slice`:

- `--project-preset-catalog <PATH>`: read the existing catalog envelope (`vendors` index/files plus `imports`) as UTF-8 JSON bytes from this explicit path.
- `--uptodate`: select `{"from_project":true,"uptodate":true}`; requires `--project-preset-catalog`. Without the switch, the supplied catalog selects `uptodate:false`.

The catalog mode requires 3MF input. It is exclusive of `--load-settings`, `--load-filaments`, and `--resources-dir`: the first two are explicit replacement modes, and the latter belongs to the old separate host-resource reporting path, not the new complete catalog supply contract. Existing 3MF restrictions on `--options` remain unchanged. Preserve plate selection, overrides, G-code output, validated processing, and optional move recording through the existing profile APIs; do not create a second slice pipeline.

The CLI reads only the requested catalog file for this supply. It does not search beside the executable, inspect an Orca installation, read environment-selected preset roots, expand Native caches, or retry a failed request with host defaults. Missing/unreadable/malformed catalog, missing named preset, missing parent, and mixed-mode requests are errors. Core must not perform filesystem I/O.

A bare request with no catalog remains exactly the old branch, including its existing reporting/resource contract. There is no automatic promotion of bare slicing to project-derived mode, and no catch-and-retry fallback between modes. Old explicit profile selection retains its current replacement semantics.

## Test-adapter supply

Use an explicit, opt-in test transport `ARES_PARITY_PROJECT_PRESET_CATALOG=<PATH>`. This is test-adapter input, not a core environment lookup or a production preset-search policy. A present path is required to read successfully; an absent variable selects the unchanged bare-request contract. Do not use preset-name/fixture-name exceptions or retry on resolution failure.

When supplied, the adapter reads these catalog bytes and invokes the existing public `ProfileSliceRequest` with `{"from_project":true,"uptodate":false}`. Its CLI counterpart passes `--project-preset-catalog` with that same file. Native receives the corresponding installed vendor tree via an isolated explicit `--datadir`; the tests preserve the staged manifest/documents, catalog, producer identity and source hashes. Native still independently parses defaults and inheritance; neither side consumes the other's settings/output as production data.

Change only input supply in the existing cases. Keep assertion bodies, fixtures, the strict whole-output comparator, normalization allowances, and Native preconditions unchanged. No exported-default or G-code snapshots enter production. An unavailable supplied preset is an error, not permission to skip a case or replace its input.

## Runtime producer and FULL profile

The documented public producer step is:

```sh
node scripts/create-project-preset-catalog.mjs --profiles "$PINNED_PROFILES" --out "$PRESET_SUPPLY"
export ARES_PARITY_PROJECT_PRESET_CATALOG="$PRESET_SUPPLY/catalog.json"
```

`PINNED_PROFILES` is the pinned Native oracle's shipped `resources/profiles` source-document tree, or the original vendor tree staged for Native. The producer reads its manifests and every referenced original document, emits the existing catalog envelope, and stages those same documents under `PRESET_SUPPLY/native/system`. It records hashes/manifest provenance. If the shipped installation contains only caches, provide the pinned original source-document tree and give its staged output to Native via `--datadir`; do not decode or substitute an exported-settings snapshot.

The catalog and staged tree are runtime artifacts: never committed, never a golden, never hand-edited. Test adapters derive the required Native data directory as the sibling `native` directory of the supplied `catalog.json`; missing supply artifacts are errors, not a reason to search elsewhere. Before relying on the staged tree, run at least one existing case twice through Native (current bare installation versus isolated staged `--datadir`) and require unchanged complete output under the existing comparator.

The FULL profile must set `ARES_PARITY_PROJECT_PRESET_CATALOG` from this producer. Supply applies uniformly through the shared adapter, never by fixture or preset name. When the key is absent, original cases run in bare mode and may fail normally: they are not skipped, passed, or excused because the key is missing. Assertions, fixtures and comparator stay unchanged.

Report the pass/fail delta across the entire registered inventory with the key absent and present, not only the directed cases. Keep deliberate missing-preset errors visible; do not repair them with hand-authored catalog entries or fallback. The integration owner receives the producer command, env contract, source hashes, tested tree and complete delta ledger. Directed checks alone do not establish FULL acceptance.

## Reproducible mechanical wiring

`node scripts/rewrite-parity-project-supply.mjs` prints a read-only replacement manifest for the current base. `node scripts/rewrite-parity-project-supply.mjs --apply` applies only qualified callee/CLI-constructor substitutions, asks `cargo fix -p ares-cli --test orca_parity --allow-dirty --allow-staged` to remove newly orphaned imports, runs rustfmt, and rejects unexpected changed paths, non-mechanical code changes, or files reaching 400 LOC. ROOT `orca_parity.rs` is outside the scanned directory and is never edited. Helper logic lives behind the existing `runner::application` namespace so the 399-line runner registry need not grow.

Integrate the logic first. Apply the generated caller patch last, on the accepted integration base after other pending test changes land. Regenerate from the script instead of manually rebasing the old caller patch. During development, prove it only on the separate verification branch. Put the generated caller changes into exactly one mechanical commit containing only callee/constructor renames and orphan-import removal; keep helper, CLI, producer, script and documentation logic in other commits.

The validator independently rustfmt-normalizes the exact rename-only expected text before comparison, preserving literal-token identity. It explicitly formats selected include fragments as well as ordinary modules; Cargo's module discovery alone does not cover lexical includes. Source partitions retain their original module scope and test IDs.

Before/after that mechanical commit, run the entire FULL inventory with the supply key absent on the same logic/dependency tree and require identical pass/fail IDs, not just equal totals. Keep newly added test IDs in both runs; separately identify any differences from an older integration inventory. Then run with the key present and retain the complete pass/fail delta. Absence never skips or passes a case. This is an integration acceptance gate, not a claim established by directed checks.

## RED and verification gates

Before coding the CLI flag, add a runnable public CLI RED after a successful Native command; unknown flag/unsupported mode is a request-boundary failure, not a compile-error RED. Retain negative cases for unreadable catalog, incomplete supplied catalog, and conflicting modes, plus an unchanged no-catalog control. Existing four core Native-first REDs and their original receipts remain separate.

Implementation modules belong in `crates/ares-cli/src/` for filesystem/argument work and the existing nested parity runner modules for test input supply. Split handwritten modules before 400 LOC. New adapter code must delegate refresh/placement/slicing to core, not reconstruct effective settings.

Develop the owned production range on the accepted merged integration base. For breadth, create a separate owned verification branch/worktree containing the approved multiline and cubic source dependencies; do not mutate their approved refs or replay dependencies already merged in that base. Inventory the requested three multiline and six footer-only cases by exact test names and input hashes, record any overlap explicitly, then run both Native modes with the explicit supply transport. Report the actual tested combined tree separately from the production tip. No FULL run or whole-suite acceptance is claimed by this slice.
