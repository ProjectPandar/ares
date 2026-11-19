# Selected-plate repetitions

Upstream: OrcaSlicer v2.5.0 `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

- `src/libslic3r/PrintConfig.cpp:12940–12944` declares signed `--repetitions COUNT`, default `1`. `src/OrcaSlicer.cpp:4938–4959` treats counts ≤1 as no-ops, rejects active repetition for slice-all or an invalid plate, and requests `COUNT - 1` additional copies. `:5299–5327` resolves plate/global sequence and skips repetition in spiral mode unless printing by object.
- `src/slic3r/GUI/PartPlate.cpp:4785–4839` (`PartPlate::duplicate_all_instance`) copies model objects from the selected plate membership, retains the instance/volume data, and names copied objects with numeric suffixes. `src/OrcaSlicer.cpp:5631–5677,5814–5853,5964–5989,6095–6268` prepares and packs the copies on the selected plate, keeps the placed subset for a single source instance, reduces the request for a group that does not fit, and restores the original if even one repetition does not fit.
- `ares-cli` owns argv, status, and files. `ares-core` owns the model-copy operation, admission against the loaded plate set and effective config, and the arrange/retry policy; use the existing Rust rewrite of `libslic3r/Arrange.cpp` instead of a second packer. Run after loading/configuration and before the selected-plate print state. The existing slice command is the adapter for this independent branch: `ares slice --plate N --repetitions COUNT` corresponds to Native `--slice N --repetitions COUNT`; `--plate 0` or an omitted plate with active repetitions is not a selected plate. Plate/outputdir routing itself belongs to the separate Native plate-dispatch change.
- Scope: saved FFF 3MF, one selected slice output, including unchanged slicing at counts ≤1. No new archive output, clone/skip-object argv, cached slice-data, arrange modifiers, or direct STL repetition. Mixed transform/export ordering is not part of this change. Existing project loading and the plater data structures are reused as the adapter for the upstream model/arrange boundary, not as a new pipeline.

## Integration overlaps

- `crates/ares-cli/src/args.rs`: slot beside `SliceArgs::plate`/`orient` intersects the transform flags/parser. Keep repetition as a signed count and preserve first-key ordering for the independent volume actions. The settings-export command reshapes the top-level `Cli`/`Commands`; retain that dispatch and keep the repetition flag on the slice arguments only.
- `crates/ares-cli/src/main.rs`: `run_slice` preflight and both `CliProjectSliceRequest` constructors intersect the transform preflight/fields. Preserve those transforms and run repetition at the selected-plate arrange stage. `main` also has the settings-export dispatch; do not route exports through repetition. With Native plate dispatch, take the selection from the resolved Native request and reject all-plates before slicing any view.
- In core, reconcile request/source wiring and the pre-slice action call with the transform plumbing. Share the selected-plate CLI arrangement setup rather than keeping two independent packer adapters.

## Public behavior checks

Compare Native and Ares CLI exits, result reports and full ordered G-code under the official and seeded oracles. Cover a successful placement, nonpositive no-op, slice-all rejection, and fit fallback. Do not commit Native outputs. Check the affected slice and arrangement inventories before and after, with no new failures among previously passing cases.
