# File-version admission for positional BBL 3MF CLI inputs

## Native contract

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`)
checks the producer version in the shared positional input loop in `CLI::run`
(`src/OrcaSlicer.cpp:1723–1779`). `Model::read_from_file` returns the BBL-family
flag and file version; the CLI checks input order first, then rejects a first
BBL-family 3MF when its `(major, minor)` exceeds that of `SoftFever_VERSION`
and the override is false. Patch, prerelease and build metadata are not part
of this comparison. Rejection precedes post-process checks (`:1799–1811`),
external preset reads and action dispatch. Neither a later BBL input nor an
otherwise invalid project is made valid by the override.

`_BBS_3MF_Importer::_handle_end_metadata` recognizes `Application` values
starting with `BambuStudio-` or `OrcaSlicer-`, and a valid `OrcaSlicer` version
metadata field also marks the file BBL-family
(`src/libslic3r/Format/bbs_3mf.cpp:4171–4187`). The importer prefers a parsed
`OrcaSlicer` field to the version parsed from `Application` (`:1537–1550`),
using the root importer's metadata for split models (`:1861–1875,1900–1947`).
This is not the 3MF package format number or a version from preset JSON.

`CLIMiscConfigDef` defines `allow_newer_file` as a Boolean defaulting to false
(`src/libslic3r/PrintConfig.cpp:13046,13247–13250`). `DynamicConfig::read_cli`
accepts the bare option and explicit values (`src/libslic3r/Config.cpp:1813–1821,
1900–1907`). `CLI::setup` puts only action and transform options in their
ordered lists (`OrcaSlicer.cpp:8204–8235`); this flag is neither. `CLI::run`
reads it once (`:1613–1615`), not as a slice/export action or a condition on
one. In particular `--export-settings -` excludes incompatible actions and
transforms (`:1451–1467`), not this miscellaneous flag.

The rejection is `CLI_FILE_VERSION_NOT_SUPPORTED` (`src/libslic3r/Utils.hpp:67`,
`-24`, process status `232` on Linux). `record_exit_reson` writes the existing
unsupported-3MF-version message (`OrcaSlicer.cpp:218,535–554,1777–1779`), plate
index zero and no completed plates. The report goes in `--outputdir` if set,
otherwise the working directory, independently of the artifact's filename.
At default logging the diagnostic is in the report, not stderr.

## CLI coverage and Ares destination

Native reaches that same gate for every supported action that takes the
positional 3MF through the main input loop, regardless of which action comes
first. Port it for the following Ares CLI adapters, including their existing
preset and selection variants:

| Positional 3MF route | Native after the input gate | Ares admission seam |
| --- | --- | --- |
| G-code, selected plate or all plates; embedded, replacement, catalog or retained-refresh presets | `OrcaSlicer.cpp:6441–6445`; preset handling begins after the input loop (`:2081–2105`) | `direct_project.rs::run` and its replacement branch; pass the policy through `PlateOptions` to `slice_plates.rs::run` from both CLI forms. `main.rs::run_slice` without a selection already passes the real flag; keep that check. |
| `--export-3mf`, sliced or unsliced | `OrcaSlicer.cpp:6423–6425,8081–8086` | `export_3mf.rs::run_direct`; sliced already uses `CliSavedSlicedProject::admit` with the real flag, so retain it. Admit the unsliced saved input before reading replacements or doing export work. |
| `--export-settings` to a file or stdout | `OrcaSlicer.cpp:1451–1467,6326–6334` | `settings_export.rs::run`; replace the unconditional bypass at its single open and report version refusal through the common CLI mapping before preset reads or writes. |
| `--export-stl` / `--export-stls` | `OrcaSlicer.cpp:6398–6412` | `export_stl.rs::run`; gate the first positional BBL archive before mesh export/preset reads, without changing file-order handling or writer behavior. |
| Positional 3MF with `--assemble` (not `--load-assemble-list`) | `OrcaSlicer.cpp:1723–1779,4911`; list construction takes the separate `:2011–2038` branch | `assembly.rs::slice` at the positional archive boundary, before preset reads. Leave list/STL assembly alone. |

The plain top-level path currently calls `CliPresetProject::open(&bytes, true)`
(`direct_project.rs:111–120`); `true` means **allow** a newer file, not **check**
it. Its replacement path calls `admit` (`:288`), which also hardcodes
`open(input, true)` (`profiles/entry/cli_presets.rs:34–39`). The shared selected
or all-plates route hardcodes the same bypass (`slice_plates.rs:80`), even
when reached from the `slice` subcommand with a parsed flag. Thus the witnessed
difference is a skipped check, not a different metadata source. Direct sliced
export passes the real value (`export_3mf.rs:100`), inverts it for the loader
(`project/export/sliced.rs:118–127`), and rejects by default.

Remove the direct clap `requires_all = ["selection", "export_3mf"]`
(`args.rs:91–101`). Retain the action/destination requirements for the actions
themselves, and accept the same bare and explicit Boolean forms on the routes
above; `=0` does not override admission. Use the existing metadata policy in
`ares-core` (`project/preset_version.rs::check_identity`, reached via
`LoadedProfileProject`), not a second version parser. Where the route already
admits an archive, reuse the single-open load; for adapters without that load,
reuse the existing metadata-only check over the same input bytes without a
second full import. Use `failure/mapping.rs::slice_error_to_failure` for the
existing `FileVersionNotSupported` exit, diagnostic and zeroed report. Apply
the comparison before presets, slicing and output side effects; the override
skips only this comparison.

## Verification

Use an existing synthetic valid saved model with a small printable plate.
Change only the root producer metadata to a version newer than 2.5 (e.g.
`OrcaSlicer` `2.6.0` with an older `Application` version). Do not source any
fixture from a user project. Reuse that input across the public CLI seams;
use matching Native argv for top-level routes and the established Native
counterpart for Ares-only subcommand spellings.

- Assert exact Native/Ares exit status, stderr and stdout, and full Linux
  `result.json` with the existing wall-clock normalization on rejection.
  Require `232`, empty stderr, and no G-code, archive, mesh, or settings
  artifact as appropriate. Include the previously bypassed selected/all-plate
  and replacement paths, and the export and positional assembly seams above.
- For the same bytes, compare the omitted flag and `=0` rejection with the
  bare and `=1` bypass. Verify successful admission with the expected
  artifact on supported routes; compare their output with the existing
  route comparator where covered. Include settings stdout as well as a file.
- Control the version source and boundary with no Orca tag (BBL `Application`
  fallback), an older Orca tag overriding a newer `Application`, a same-minor
  newer patch, and a non-BBL application. A missing or incompatible external
  preset must not mask the version refusal on the valid newer fixture.
- Run committed pre-fix regressions and exact-tip verification with both
  Official and Seeded Native, then pair the affected existing CLI cohort at
  base and tip (including previously checked sliced export), and run both
  Clippy gates. Do not relax an existing comparator or skip selected cases.

## Deferred behavior

No new Native-only actions (such as `--export-obj`, `--info` or inspect) or
positional combinations are added to Ares. No new input-order, malformed
archive error precedence, package-format version, normative, compatibility,
geometry, preset, or export behavior is ported here. No version gate is added
to non-CLI imports, browser/WASM loading, `--load-assemble-list`, or STL/OBJ
sources.
