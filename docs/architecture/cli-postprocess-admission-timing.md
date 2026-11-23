# Check embedded post-process settings before retained preset refresh

## Upstream boundary

In OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`src/OrcaSlicer.cpp:1758–1760` imports the model and config and enters the
BBL-family 3MF branch. `src/libslic3r/Model.cpp:472` delegates to
`Format/bbs_3mf.cpp::load_bbs_3mf` (`:9569`); `PrintConfig.cpp:5985` defines
`post_process` as a `ConfigOptionStrings` option. The CLI applies
`normative_check` to this **imported** config (`OrcaSlicer.cpp:1800–1809`). It
rejects a non-empty vector with `CLI_POSTPROCESS_NOT_SUPPORTED` (`-19`).
The check is on the number of entries, not their contents: an embedded
`[""]` also fails. A missing or empty list passes that check.

The later retained-preset reads and merge (`OrcaSlicer.cpp:2539–2817`,
`:3165–3199`, and `:3389–3393`) do not re-run it. For instance, a saved Prusa
MK3S project with an embedded `[]` is admitted, then the stock process can
supply `[""]` from `resources/profiles/Prusa/process/process_common_mk3.json:57`.
The CLI does not execute post-process scripts: the call at `:7236` is commented
out. Neither changing the option definition nor executing scripts is part of
this change.

## Ares admission sites and source modes

All checks use `ares-core/src/project_slice/validated.rs::validate_post_process`.
Keep its vector-length test and `ares-cli/src/failure/mapping.rs` error
mapping (exit/report code `-19`, failure at plate index zero, no diagnostic
written to CLI stderr by that mapping). On the current Rust boundary:

| Site in `ares-core/src` | Source mode and data examined | Disposition |
| --- | --- | --- |
| `project/load/profile_project.rs::LoadedProfileProject::load` | `CliPresetProject::admit` imports 3MF bytes before opening external presets. Used by retained saved CLI admission, explicit preset files and catalog entrypoints. Non-3MF inputs are not imported by this check. | Keep the check when normative checking is enabled. It corresponds to the Native import check. |
| `profiles/project_refresh.rs::ProjectPresetRefresh::load` | Project-derived refresh over 3MF bytes, using an already admitted model when available or loading the embedded config itself. Native retained sources and project-derived catalog sources use this path. The plater entry does not request this CLI admission. | Keep the pre-merge check. It protects the public retained-refresh entry even when invoked without prior CLI admission. |
| `project/domain/profile_input/three_mf.rs::Project::from_3mf_with_profiles` | Profile-backed 3MF bytes and profile-backed assembly archive input. `project_slice/profile_entry.rs` passes the CLI validation/normative flags before explicit preset overlays; a pre-admitted project can reach it too. STL or an assembly model does not use the 3MF entry. | Keep the embedded check. Do not alter sharing or ordering for profile-backed input. |
| `project_slice/state.rs::load_project_source` | Unprofiled input selected by `ProjectInput::load`: saved CLI bytes, other validated byte APIs (including processed and archive output), and unprofiled model/assembly/options sources if the validation flags request it. The state check runs after the load and therefore sees the result of retained refresh, not the embedded data. | Narrow this late check for the Native retained source. That source has already performed the import check in `ProjectPresetRefresh::load`; rechecking the merged config has no Native counterpart. Leave the check in the other source modes unchanged, including those where it is the only check. |

The Rust destination is the relationship between `ProjectPresetRefresh::load`
and `project_slice::state::load_project_source`. Reuse their existing import
and refresh boundaries; do not drop the blanket state check for sources that
do not run the earlier check or decide admission based on the merged list's
contents. The change applies to the Native saved retained-preset entry through
`slice_project` / `slice_cli_project_plate_with_refresh`, and to callers that
reuse that same refresh source. Plain saved 3MF without opt-in, the Ares-only
catalog path, explicit replacement precedence, STL/assembly entry policies,
and any discrepancy around the existing non-BBL classification are separate.
No public interface or slicer pipeline changes are needed.

## Public CLI acceptance

Create a synthetic BBL-family saved 3MF with matching stock Prusa MK3S presets.
Derive both inputs from the same repository fixture by setting only its
embedded `post_process` list: `[]` in one, `[""]` in the other. Stage the
Ares executable under `installation/bin` with the pinned resources at
`installation/resources`. Use Ares `slice --uptodate -o out/plate_1.gcode` to
exercise the existing non-selection CLI entry. Run Native at test time with
`--slice 1 --outputdir out --uptodate` on the same inputs; those output switches
select the equivalent single-plate result.

- `[]`: both must succeed. Compare ordered G-code bytes after normalizing only
  the generator line; require the Native and Ares footer to contain the
  resolved `; post_process = ""`. This verifies that admission did not pass
  merely by skipping the installed catalog or stripping the merged option.
  The unfixed Ares executable must fail with `-19` on this input.
- `[""]`: both must reject before output; compare process exit status, stdout,
  stderr, and the `result.json` fields (apart from elapsed wall time). Expect
  `-19` and no G-code. The existing embedded non-empty-list controls also
  remain part of the focused regression run.

Run both cases and the existing normative-check, embedded-post-process,
load-settings and retained-refresh controls with official and seeded references.
No Native output is stored in the repository.
