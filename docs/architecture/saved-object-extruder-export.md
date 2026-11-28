# Preserve the loaded object extruder in saved sliced 3MF export

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`src/libslic3r/Format/bbs_3mf.cpp`:

- `_BBS_3MF_Importer::_load_model_from_file`, lines 2272–2280: object
  metadata other than `name` and `module` is deserialized into the object's
  config. `extruder` is a 1-based logical filament selector, not a physical
  tool index.
- Same function, lines 2376–2384: the maximum selector is the size of the
  loaded `filament_settings_id` vector, or `int`'s maximum when that option
  is absent. The initial selector is `0` when no object option is present.
  `extruder_id == 0 || extruder_id > max_filament_id` sets the object option
  to `1`. With two loaded filament slots this preserves `1` and `2`, and
  turns `3` into `1` before slicing.
- Lines 2386–2394 distinguish the lone-volume erase from the multi-volume
  policy. Keep the existing lone-volume erase; it does not clear or replace
  the parent object's selector.
- `_BBS_3MF_Exporter::_add_model_config_file_to_archive`, lines 8323–8326:
  serialize the live object config into `Metadata/model_settings.config`.
  This writes the imported selector after the default/range rule, not the
  original XML token or a constant.

## Ares mismatch

A two-filament, single-part saved archive sliced with
`--slice 1 --export-3mf` yields:

| Saved object selector | Native archive | Ares archive |
| --- | --- | --- |
| `1` | Object `1`, no part selector | Object `1`, no part selector |
| `2` | Object `2`, no part selector | Object `1`, no part selector |
| `3` | Object `1`, no part selector | No archive; process status `243` |

In `crates/ares-core/src/project/export/saved.rs`,
`from_saved_project_with_extras` uses `geometry::cli_object_metadata`, which
always writes `1`; it does not read the selector in `ProjectObject`'s region
config. For `3`, `effective_config/selector_validation.rs` rejects the raw
object selector against the logical filament count before export can finish.
The `SliceError::InvalidInput` detail is `invalid Orca option extruder`.
`crates/ares-cli/src/export_3mf.rs` reports it through
`CliFailure::project_export` (`failure/export.rs`): signed return code `-13`,
`Failed exporting 3mf files.`, and Unix process status `243`.

## Port and limits

- Put the saved object default/range rule at the saved 3MF import boundary
  in `crates/ares-core/src/project/load.rs` and `project/load/assemble.rs`,
  where object config becomes a `ProjectObject`. Use the imported saved
  filament ID list, before later caller overrides or profile materialization,
  as the bound. Do not change the raw model-settings document or replace
  effective selector validation with clamping. Leave unrelated validation,
  including negative selectors, as-is. This loaded object also feeds plain
  project G-code slicing and the browser build, so they see the same
  default/range result before slicing.
- In `project/export/saved.rs`, the full saved CLI sliced export projects
  the live object selector from the same loaded domain object used for
  slicing. Replace its use of the constant metadata record on this route
  only. Do not read the selector from the lone part, from G-code, or from
  the raw object XML at serialization time.
- Preserve existing archive and model admission constraints. This change
  covers the already-admitted translated ordinary saved model path; it
  does not widen affine admission, multi-volume selector handling, or
  non-Bambu-family/plain mesh import. Unsliced export, min-save archive
  behavior, STL export, provenance, mesh repair counts, and other object
  metadata keys are unchanged.

## Verification

Generate a synthetic saved project in the parity test from the suite's
ordinary model and preset helpers. Use two loaded filament slots and one
normal part; change only the saved object selector across inputs `1`, `2`,
and `3` after making the same translation adjustment in each input. Do not
use archived outputs or inputs from a local run. Run the same saved sliced
export command with Native and Ares. Compare `Metadata/model_settings.config`
byte for byte, including the exact object and part `extruder` metadata.
Each case must produce an archive. Confirm Native's expected `1`, `2`, `1`
results and the absence of a part selector so the fixture exercises the
intended input.

Also slice the selector-`3` fixture to plain G-code with the same command
on both CLIs, using the existing ordered-byte comparator. This checks the
shared importer outside the archive exporter.

On the test-only commit, the `1` control passes, `2` fails on object metadata,
and `3` fails at Ares export and plain slicing. Verify in both Official and
Seeded modes before implementation. On the implementation tip all four
must pass in both modes, alongside the existing saved selector and mesh-stat
cases. Paired scoped base/tip runs cover saved export, project load, and
project G-code slicing suites without changing their comparators. Require
no passing-to-failing cases and no selected skips.
