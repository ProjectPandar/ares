# Config substitution warnings after positional 3MF load

## Native boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) resolves
positional inputs in `src/OrcaSlicer.cpp:8203–8217` before doing work that can
change the process directory. `src/libslic3r/utils.cpp:1577–1588`,
`resolve_cli_input_path`, leaves an already absolute input spelling alone.
It completes a relative path against the invocation directory, lexically
normalizes it and uses platform-preferred separators. This is not canonical
filesystem resolution; it does not dereference symlinks.

`OrcaSlicer.cpp:1723–1758` passes a `ConfigSubstitutionContext` into
`Model::read_from_file`; only the first positional 3MF receives `LoadConfig`.
The project config reader in `src/libslic3r/Format/bbs_3mf.cpp:2791–2811`
passes that context through the embedded JSON load. `src/libslic3r/Config.cpp:600–615,666–711`
applies legacy translation before deserialization and records an unrecognized
enum value that is replaced by the option default. A legacy translation is
not a substitution.

The report is at `OrcaSlicer.cpp:1992`, after the model load and checks such
as file version (`:1775–1780`), normative post-process admission
(`:1800–1812`), and printer technology (`:1982–1992`). Failed file discovery,
load, or admission **before** that call does not report substitutions merely
because an embedded config member was readable. At `:520–534` Native reports
one stderr line per substitution, in order:

```text
warning: <file>: value "<old>" for <key> is not valid, using "<new>" instead
```

The same file spelling and payload (`file`, `key`, `old_value`, `new_value`)
go into the `config_value_substituted` record (`:525–530`). The Linux
`record_exit_reson` writes the accumulated records into `result.json` under
`--outputdir` or in the working directory (`:535–586`). Its late error and
success callers both use the accumulated records, e.g. invalid plate selection
(`:5291–5296`), slice preflight (`:6562–6573`), project export failure (`:8081–8086`) and final success
(`:8126–8136`). Per-plate callback warnings (`:7142–7178`) are later and
must not be moved ahead of the load warning.

## Route audit on `b340ddbb0`

Each cell gives **substitution stderr / Linux `warnings[].file`**. `abs` is
Native's lexically completed relative path; `raw` is the input spelling;
`—` means no substitution in that channel and `no report` means no
`result.json`. These are substitution channels only, not all CLI output.
The Native column applies to all included rows: `abs / abs` on success and
on failure **after** the report; `— / —` when Native writes a report for a
failure **before** the call. The pinned official-lane synthetic probes used
`inputs/sub/.././project.3mf` and a `cross-zag` embedded enum value. Source
inspection covers earlier branches and fallible paths not exercised in the
probe.

| Positional saved 3MF route | Ares success | Ares failure after Native's load report | Ares seam |
| --- | --- | --- | --- |
| Direct single-plate `--slice 1` | `— / raw` | Missing external presets: `— / no report`; generic typed slice error: `— / —`; path-conflict and late critical checks can carry `raw / raw`. Late legacy I/O errors can also be `— / no report`. | `direct_project.rs:179–275`, `failure/slicing.rs::check_direct` |
| Direct selected plate `--slice N` | `— / raw` | Out-of-range positive selection: `— / —`; other paths are the same as `--slice 1`. | `direct_project.rs:49–53,102–110,209–275` |
| Direct all plates `--slice 0` | `raw / raw`, once per project | Selection/plate preflight or publish failure through batch adapter: `raw / raw`; resource/preset/action errors before that adapter can drop both channels. | `direct_project.rs:70–99`, `slice_plates.rs:56–288` |
| Sliced `--slice 1 --export-3mf` | `— / no report` | Blocked archive destination: `— / —`; same omission on typed late export refusal. | `export_3mf.rs:111–171`, `failure/export.rs::project_export` |
| Unsliced `--export-3mf` | `— / no report` | Blocked archive destination: `— / —`; same omission on typed late export refusal. | `export_3mf.rs:80–106,148–171`, `failure/export.rs::project_export` |
| `--export-settings` to file or stdout | `raw / no report`; stdout itself remains settings JSON | Missing external presets after load: `— / no report`. Blocked settings destination: `raw / no report` (Native logs the write failure but exits zero with `abs / abs`, `Config.cpp:1546–1555`). | `direct_stl.rs:30–39`, `settings_export.rs:17–68,71–122` |
| `slice` subcommand, positional project, single-plate `-o` | `raw / raw` | Typed slice failure through embedded/replacement adapter: `raw / raw`; errors before those adapters or legacy writes can drop both. | `main.rs:162–203,274–285`, `project_presets.rs:163–224,265–325`, `load_settings.rs:54–116` |
| `slice` subcommand, positional project, selected/all `--slice` | `raw / raw`, once per project | Selection/plate preflight or publish failure: `raw / raw`; earlier setup errors as in direct batch. | `main.rs:142–145`, `slice_plates.rs:56–288` |

A missing input, a load that returns no admitted project, or a disallowed
project version is not a late failure. Current adapters do not report
substitutions on those paths (they may differ in whether they write an empty
report). The direct and subcommand `empty-plate` smoke probes are **not**
late parity cases: Native rejected that load with `-6` and no substitution;
the subcommand proceeded to `-50` in Ares and reported `raw / raw`.
Do not use that probe to justify reporting before Native admission or to
change either engine's exit family. The existing absolute empty-plate
subcommand test uses the parity runner rather than that smoke invocation.

## Included work

Bring every included route to the shared reporting contract, including
explicit/retained preset paths **for the positional project's** substitutions,
not warnings read from preset files themselves. Establish the positional load
report only after the equivalent admission checks and before action-specific
work. Do not treat the end of slicing/export as the load boundary. Retain
its typed ordered records through success and every later failure, including
through output writes. Emit stderr once, before later diagnostics, rather
than duplicating the existing `report_success`, `check_direct`, and
`with_config_substitutions` output. The current adapters suppress reporting
on the pre-report `-19` family (`failure.rs:225–248`); keep the admission
boundary, not a blanket rule to append warnings to every error.

Reuse the existing formatter and `CliWarning::ConfigValueSubstituted`
serialization. Derive warning identity from the positional argv and the
invocation cwd, independently of opening the input or deriving an output
stem. Reuse the admitted project's typed substitutions; sliced/unsliced
export must make them available from its load/admission without adding a
second independent parse of the archive. Reading only the settings member
via `project_config_substitutions` (`ares-core/src/project/load.rs:61–70`)
is insufficient to establish that Native reached its reporting point.

For Linux, add the missing project export/settings success report using the
existing writer and append the retained warnings to late failure reports.
Where a late path has no writer, deliver the warning/report without
reclassifying the action failure. For these export routes, use Native's
report location (independent of the archive/settings destination). Do not
send diagnostics or the report to settings stdout. Do not change action
admission, result codes, archive/settings bytes, G-code, or unrelated report
fields merely to force whole-report parity; the settings destination-write
exit policy is independent of the substitution report.

## Deferred, distinct seams

- Saved 3MF to `--export-stl`/`--export-stls`: Native does share the load
  report and then dispatches STL export (`OrcaSlicer.cpp:6398–6413`); the
  synthetic saved project successfully reported there. Ares refuses the
  saved input before importing settings: `project/export_stl/geometry.rs:110–123`
  admits plain-geometry archive members only, and `export_stl.rs:92–115`
  invokes that importer. That route needs a distinct saved-project STL
  importer admission change, not a fix to an existing config-load report.
- Assembly archives are not positional inputs. Native takes the other
  branch at `OrcaSlicer.cpp:2011–2016`; applying the positional path contract
  to an assembly-supplied path in `assembly.rs:228–243` would conflate two
  input sources. URI, STL inputs, and actions Ares does not admit for saved
  3MF are not new import/action work here.
- External preset substitutions, new substitution rules, and per-plate
  callback warning semantics are separate. Preserve their existing behavior
  and keep load records before callback records in the same result.

## Synthetic verification

Use independently printable synthetic saved inputs (one plate and multiple
plates). Replace embedded `support_ironing_pattern` with `cross-zag`;
`PrintConfig.cpp:7985–7995` supplies the `rectilinear` default. `zig-zag` is
translated by `handle_legacy` (`:9931–9937`) rather than substituted. Include
an unchanged or legacy-translated input as a no-substitution control.

- Run Native and Ares from the same invocation directory with the same
  relative dot-segment path and, separately, an already absolute path with
  dot segments. Keep output separate between runs. Use equivalent Native
  `--slice` invocations for the Ares `slice` subcommand. Verify every success
  row, including settings file and stdout, selected and all plates. Require
  the expected outputs, exactly one load report per input, and no
  duplicate load records on batch runs.
- Verify late failures with positive out-of-range selection, all-plate
  preflight with a nonprintable later plate, blocked 3MF output directories
  at the destination file (both modes), and a missing external settings
  file after load. Also cover a before-report rejection (a newer version
  without opt-in or invalid project) with no substitution in either channel.
- Compare exact warning stderr and the complete ordered warning records
  with no masking of path, payload or ordering. Include a case with a later
  diagnostic to check ordering; use elapsed-time normalization only where
  the rest of the report is already comparable. Check both Native lanes at
  the frozen parent, test-only commit and corrected tip. Pair adjacent
  direct slicing, export, result report and ordered warning coverage to
  check exit behavior and artifacts outside the warning change.
