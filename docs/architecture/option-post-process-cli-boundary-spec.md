# post_process: bounded CLI admission and execution-ownership spec

This is a source-cited admission/data rewrite slice, not a new Ares
script-running feature. The original spec audit was entirely offline.

## Upstream owners and distinct meanings

1. `PrintConfig.cpp:5068-5079` defines `post_process` as `coStrings`, default
   empty list, serialized/multiline; tooltip describes absolute script paths,
   arguments and config environment. It is not a finite enum/numeric domain.
2. `OrcaSlicer.cpp:1438,1466-1468` defaults `normative_check=true` and reads its
   option. The embedded-3MF config guard `:1682-1690` checks vector **size**,
   not whether trimmed script text is nonempty. Any nonempty list, including
   `[""]`, records `CLI_POSTPROCESS_NOT_SUPPORTED` / -19 with plate_index0
   and exits before Print/export. Diagnostic is `:131`; channels/result use
   `:398-460`. This guard precedes external settings composition
   (`:1953-3495`); it is not a check of every final merged script value.
3. The CLI export site `OrcaSlicer.cpp:6247-6248` has its external
   `run_post_process_scripts` call **commented out**. An exported config
   containing a script is not proof that the Native CLI executes it.
4. External execution is actually owned by
   `libslic3r/GCode/PostProcessor.hpp:12-26` and `PostProcessor.cpp`:
   - POSIX `run_script`, `:159-185`, invokes `$SHELL` (default `/bin/sh`),
     appends a quoted G-code path, captures stderr and waits for exit status.
   - Windows `:63-153` handles command-line quoting, `.pl`/`.bat`, process
     creation/waiting and exit status separately.
   - `run_post_process_scripts`, `:238-377`, checks scripts/source file,
     optional `.pp` copy, config environment and `SLIC3R_PP_HOST` /
     `SLIC3R_PP_OUTPUT_NAME`, splits CR/LF lines and trims/skips empty lines,
     runs in order, handles nonzero status/deleted files, `.output_name`
     replacement and cleanup. An all-empty nonempty vector differs here
     from the normative CLI guard; do not conflate the two predicates.
   - Real callers are native GUI/export/upload adapter orchestration:
     `BackgroundSlicingProcess.cpp:246,793-819,928-935`, including a copy to
     protect the viewer's mapped original output. The disabled CLI call
     cannot be used as a live runtime oracle for these GUI semantics.
5. `GCode.cpp:2174` invokes `m_processor.finalize(true)`;
   `GCode/GCodeProcessor.cpp:981,2685-2716` runs the **internal G-code
   statistics/progress post-process pass**. This is not external scripts.
   Ares's processor `post_process: bool` belongs to that separate boundary.

## Non-negotiable Rust destination and ownership

- `ares-core`: pure option representation, configuration composition and
  typed admission result only. No filesystem, `std::process`, shell, child
  process, or mutation of process environment; must work on browser WASM,
  Windows, macOS and Linux. Existing `PostProcessNotSupported` is a temporary
  CLI-compatibility admission shell in the validated loading routes, not an
  execution facility or a reason to reject every pure byte API.
- `ares-cli`: owns filenames, filesystem I/O and typed-error-to-exit/channels
  mapping. **If** an external script runner is ever added,
  its only destination is a native CLI filesystem adapter, after exported
  file materialization; not core, not WASM and not an implicit invocation of
  untrusted profile content. This spec adds no such runner.
- Current slice's test destination:
  `crates/ares-cli/tests/orca_parity/option_coverage/post_process.rs`, with
  narrowly owned classification hook if justified by runtime proof; reuse
  existing Native runner/adapters. Any source-admission disparity is resolved
  before production changes. Core API/variant changes require a
  separate decision. Keep other option hooks untouched and files <=400 LOC.

## Ares scaffold audit (observations, not runtime findings)

The combined 3MF + load-settings tracer is excluded from this slice. Native
-19/237 versus Ares's explicit unsupported-route exit1 is real runtime
disagreement; it is **blocked-by load-settings-3mf**, not permission to alter
that entry guard here. This slice closes ordinary embedded nonempty / `[""]` /
`[]` CLI witnesses first.

- `options/process_options/print_source.rs:32` stores `OrcaStrings`, empty
  default; wire/export and registry retain the option. No execution found.
- `project_slice/validated.rs:12-25` checks embedded raw list length and
  returns `PostProcessNotSupported`; `state.rs` gates it on validate_print.
  `project/domain/profile_input.rs:62-66` intentionally checks embedded
  config before profile overlay on validated routes.
- `ares-cli/src/failure.rs` maps the typed variant to -19, empty stderr,
  plate_index0 and exact JSON diagnostic; existing runtime early-exit and
  eight stock CLI cases already exercise `[""]` / Prusa rejection.
- `profiles/entry.rs:181-234` constructs the load-settings ProjectSource
  with `validate_print=false`; CLI's loaded-settings route calls it. This
  could leave embedded nonempty scripts unguarded relative to the Native
  `:1682` admission phase. This is **unproven**, not a fix in scope: test
  the identical 3MF with/without external settings first. Conversely a
  script introduced only by external settings must not be rejected merely
  because the final composed list is nonempty if Native admits it.
- Existing option sweep's baseline-only post_process row does not prove
  either execution or nonempty-vector admission; do not promote it to
  exhaustive PASS over arbitrary executable strings.

## Included bounded E2E witnesses

Use one eligible rectangular-bed stock baseline and the existing 3MF member
adapter. Retain original export and verify model/config ZIP payloads except
exact requested option edit. Ares/Native receive identical input bytes.

| Source of value | Request | Required observation |
|---|---|---|
| embedded absent / `[]` | ordinary 3MF CLI | actual Native success and whole output; schema/default/list proof |
| embedded `[""]` | ordinary CLI | actual Native -19/Unix237, exact stdout/stderr/full JSON, plate0, no G-code |
| embedded whitespace-only singleton | ordinary CLI | vector-size guard; do not trim into empty list |
| embedded harmless absolute command string | ordinary CLI | -19 before any command execution; no output/side-effect artifact |
| embedded two string entries | ordinary CLI | same size guard; distinguish count from empty-text semantics |
| embedded nonempty + baseline external settings | load-settings CLI | source-phase witness before overlay; actual Native/Ares outcome, stop on disparity |
| embedded empty + external settings-only nonempty | load-settings CLI | merged representation and actual export behavior; CLI script call is disabled, do not invent execution |

Commands for later no-execution witnesses must be harness-owned inert test
scripts in a temporary directory, never user/system scripts. A marker file
allows proving **absence** of execution; no inherited shell/profile commands
are trusted. This is a safety control, not a new golden. Platform-dependent
exit/result behavior follows existing CLI test contracts; Linux full JSON
receipt is not falsely claimed as a Windows/macOS receipt.

Core byte-data control uses the corresponding pure configuration/byte API,
verifies retained option values and typed admission where that API promises
validated CLI compatibility, never requires a child process. Invalid value
shapes follow existing input/deserialization boundaries; do not invent
private defensive checks or alter accepted serialization speculatively.

## Ordinary CLI witness execution

Seeded and official fresh caches: each **3/3 PASS**. Nonempty owned command and
`[""]` both match Native -19/Unix237, exact channels/full JSON/plate0,
no export and absent execution marker. `[]` matches successful complete
G-code bytes, generator-only normalization. Same authored 3MF input is used
for both slicers; every non-post_process setting and other ZIP payload is
proved unchanged. No core/CLI production changes were required.
External script execution remains INCOMPLETE, and the combined load-settings
case stays blocked; neither is promoted by these six PASS tests. The harness
reads the existing file-backed status/channel receipts.

## TDD, failure modes and evidence

Before implementation: separately committed test-first admission cases,
successful compilation, then a genuine named runtime RED. Existing PASS
or metadata-only INCOMPLETE is not production behavioral RED. If all actual
behavior already agrees, this remains a test/classification-only slice.

Failure modes to target: nonempty empty-string list treated as empty;
checking merged rather than embedded pre-overlay config; load-settings
bypassing normative admission; rewriting strings into executable commands;
accidental script execution from profiles; rejecting pure byte APIs without
source justification; confusing internal M73/statistics pass with external
scripts; wrong -19 JSON/plate/channel or a G-code export before admission.

Save input/ZIP member SHA, original export, producer identity, exact argv,
stdout/stderr/exit/full JSON, complete G-code when exported, command text
and absent-marker receipt. Only existing generator and two-integer
wall-clock rules apply. Required Native absence must named FAIL, optional
absence visibly SKIP. No frozen Native authority, prefix comparisons,
retry-to-green, fixture repair, cache relaxation or comparator changes.

Classification may record **bounded CLI admission/no-execution coverage**,
not exhaustive `coStrings` execution parity. Arbitrary commands, argument
parsing, permissions, failure stderr, script order, Windows perl/batch,
`.pp`/renaming/upload host behavior and normative_check=false are deferred.
No new adapter flag/runner is introduced to reach those deferred branches.
External process execution remains INCOMPLETE until a separate
adapter/source-runtime oracle slice supplies that boundary. Full legal
scope must not be fabricated from these witnesses.

Exit: bounded metadata meaning, all requested source-specific Native E2E
witnesses or explicit unresolved disparities, other option plans unchanged,
and existing stock -19/CLI regressions retained.
