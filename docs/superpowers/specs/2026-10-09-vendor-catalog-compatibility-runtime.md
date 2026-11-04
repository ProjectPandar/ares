# Spec: ten stock vendor catalog/compatibility runtime conclusions

Status: source-only proposal. Follows the libslic3r catalog/compatibility
boundary, not the previously observed difference between Native and Ares CLI
argv surfaces.
Historical VENDOR_INCOMPLETE rows and Native diagnostics remain non-PASS inputs
to this proposal; no public Core runtime mismatch is yet demonstrated.

## Upstream rewrite boundary / Rust destination

- Preset.cpp:809-850::is_compatible_with_printer: system-printer name/list match,
  expression/extras handling and inherited merged configuration.
- Preset.cpp:771-808::is_compatible_with_print is adjacent filament/process
  behavior; no new independent coverage claim for that predicate in this slice.
- PresetBundle system vendor loading and parent composition:4862-4921;
  Preset.cpp load-time selectability/missing-parent handling. Missing default
  process selection is distinct from a missing machine/process parent file.
- Rust: existing ares-core::profiles::{compatible.rs,catalog.rs} exposed by
  PresetCatalogHandle::{add_vendor,filter_compatible} and
  filter_compatible_presets. No new Core API, fallback registry or pipeline.

Included: load exactly the same stock vendor bundle/manifest and preserve all
profile names, default references, parent chains and explicit compatibility
lists; compare nine selected-default-process compatibility decisions and one
missing-default selection error with same-run Native conclusions.
Deferred: UI auto-selection/default-visibility state; general filament/process
expression coverage; CLI argv/export product surface; slicing/geometry/G-code
changes; malformed new fixtures, signals, profile repairs and missing-parent
scenarios not actually present in these ten rows.

## Test location and exact cohort

Use the existing **ares-cli orca_parity integration harness**, not Core unit
tests with filesystem/process behavior:
`crates/ares-cli/tests/orca_parity/runner/vendor_profile_compatibility.rs`, plus
its case table if needed and normal runner module registration. This harness
owns filesystem/Native execution; it calls only **public ares-core profiles
APIs** for Ares decisions. No Native subprocess or direct I/O is added to Core.

Selectors remain the ten rows in the audit:
- Creality K1 SE0.8: absent `0.40mm Standard @Creality K1 SE 0.8 nozzle`.
- Ratrig RatRig V-Core4 300/400/5000.8: existing
  `0.20mm Standard @RatRig` (complete parent chain), excluded by printer lists.
- iQ TiQ2/TiQ8 0.25/0.6/0.8: existing
  `0.20mm Standard @iQ TiQ2 P1 - PACF Pro Fiberthree (0.4 Nozzle)` (complete
  parent chain), excluded by printer lists. Nozzle-case differences between
  internal names and filenames are preserved, not repaired.

Reuse runner/profile_entry.rs::vendor for `{index,files}` transport, including
OrcaFilamentLibrary plus the owning vendor in the same order. Add those exact
vendor envelopes with PresetCatalogHandle::add_vendor; no synthetic catalogue
entries or aliases. Retain manifest/file hashes and selected stock metadata.

## How Native conclusions are obtained at runtime

Every admitted test first checks the shared native_environment guard. Missing
optional Native prints visible SKIPPED before artifact-root access; required
absence fails closed. Use OrcaRunner::from_env and its declared/identified
producer plus existing stages::run capture. No frozen Native record/output
or committed Native product may satisfy the test.

For nine incompatibility rows:
1. Resolve the literal machine/default process/default filament from the stock
   files using existing VendorProfiles inheritance and write_preset transport.
   Do **not** call select_printer's alternative-compatible selection: its early
   VENDOR_INCOMPLETE would prevent obtaining the requested Native conclusion.
   Preserve the original nonempty compatible_printers lists unchanged. This is
   an explicit request for the named stock default, not a stock-selector PASS.
2. Reuse the established Native load-settings/load-filaments + arrange1 +
   export-3mf invocation shape on cube10.stl, through stages::run in a unique
   case work directory. Capture the expected early failure and complete
   command/status/stdout/stderr/result.json/00000.log there. These failures do
   not publish a successful export cache entry or create a3MF; no new success
   cache schema or cli_result recipe modification is needed.
3. Admit an incompatible conclusion only if that **same invocation** has an
   actual non-signal process exit239, result return_code -17 and log diagnostic
   from the source-owned2652 `process not compatible with printer` branch.
   Extract these from the fresh capture; the assertion's reference decision is
   derived from this Native conclusion, never a handwritten false/golden record.
   Missing/incomplete captures or a different Native failure are INCONCLUSIVE,
   not a Core RED. Record binary/resources/environment/argv/commit/input hashes.
4. Call handle.filter_compatible(machine, Some(default_process)), serialize its
   public FilteredPresets output, and determine membership of the exact selected
   process. Compare that actual membership to the Native-derived compatible
   decision. Parent/manifest loading failures are recorded separately; never
   flatten them away, ignore missing parents or substitute a process.

For K1:
1. Keep the exact missing default name from machine metadata. Do not manufacture
   a process or replace it with a0.4/0.6 alternative. Native requests its absent
   process-file path after loading the original machine/default filament,
   using the existing staged invocation/capture. Same-run exit253/-3 and the
   missing-setting-file diagnostic must identify that exact requested target.
2. Load the complete real vendor envelope and call
   handle.filter_compatible(machine, Some(default_process)). Assert a public
   typed missing-process InvalidInput naming the same missing default, derived
   from the machine metadata plus the admitted Native missing-file conclusion.
   It is **not** a missing-parent error or failed add_vendor. Core has no file
   path/process-exit contract; do not compare Core's typed name payload to
   Native's filesystem-channel text or add filesystem policy to Core.

Native stdout/stderr/result/log are retained externally for verifiability, not
copied into source. No CLI channel parity claim is made for this API comparison.
If result bytes need comparison anywhere, use only shared native_wall_clock;
this decision comparison needs no second JSON wall-clock normalizer.

## TDD, ownership and gates

Failure inventory before implementation: wrong manifest scope/name case,
wrong literal default, repaired compatibility lists, silently missing parents,
Native wrapper/cache absence, stale expectation, wrong early error (width,
input-read or signal), lost log, missing result, hand-authored false, comparing
Core missing-name with a filesystem exit code, and invoking private internals
instead of the public profiles API.

Write tests first and run required Native conclusions on
latest actual tree. Keep unchanged genuine public-API RED before any production
change. If Core disagrees, port only missing Preset/PresetBundle behavior in the
existing compatible/catalog owner; no CLI guard or unrelated pipeline work.
If all ten agree, deliver tests/docs-only closure and retain original raw stock
VENDOR_INCOMPLETE status (not valid slicing PASS).

Do not touch the cli_exit_channels/cli_result cache. #30's eight -50/-19 cases
remain independent. Final gates: strict workspace clippy/fmt, default no-ARES
workspace with visible skips, required seeded/official ten-case directed runs;
production changes also need latest-main workspace/paired gates.

Conclude with source ownership, actual run-to-commit bindings and retained
failed/inconclusive attempts. No implementation has started.
