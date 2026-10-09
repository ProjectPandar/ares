# Ten VENDOR_INCOMPLETE rows: profile and CLI audit

Analysis/docs only; no heavy job, production changes or preset repairs.
Historical stock inventory is bound to829977606/2bec. Profile/Native diagnostic
analysis used d9cb2b229 (tests/docs preparation on104599195); this document is
prepared on4e6417427. Existing Ares executable was built at304f49867, not at
104599195/4e6417427. Its SHA and that limitation are explicit in each receipt.
Current ares-cli/src/main.rs still exposes the same Slice-only argument surface.

## What is actually missing

Profile paths below are relative to `OrcaSlicer/resources/profiles`.

| Stock printer | Machine file | Default process / chain | Missing condition |
|---|---|---|---|
| Creality K1 SE0.8 | Creality/machine/Creality K1 SE 0.8 nozzle.json | `0.40mm Standard @Creality K1 SE 0.8 nozzle` | Default process file/name absent; no compatible instantiated process selected |
| RatRig V-Core4 3000.8 | Ratrig/machine/RatRig V-Core 4 300 0.8 nozzle.json | R chain below | No process compatible_printers edge to this exact printer; not a missing inherits file |
| RatRig V-Core4 4000.8 | Ratrig/machine/RatRig V-Core 4 400 0.8 nozzle.json | R chain | Same absent compatibility edge |
| RatRig V-Core4 5000.8 | Ratrig/machine/RatRig V-Core 4 500 0.8 nozzle.json | R chain | Same absent compatibility edge |
| iQ TiQ2 0.25 | iQ/machine/iQ TiQ2 0.25 nozzle.json | Q chain below | Process lists0.4 only, not this leaf |
| iQ TiQ2 0.6 | iQ/machine/iQ TiQ2 0.6 nozzle.json | Q chain | Same absent compatibility edge |
| iQ TiQ2 0.8 | iQ/machine/iQ TiQ2 0.8 nozzle.json | Q chain | Same absent compatibility edge |
| iQ TiQ8 0.25 | iQ/machine/iQ TiQ8 0.25 nozzle.json | Q chain | Default is TiQ2's0.4 process, incompatible with this TiQ8 leaf |
| iQ TiQ8 0.6 | iQ/machine/iQ TiQ8 0.6 nozzle.json | Q chain | Same absent compatibility edge |
| iQ TiQ8 0.8 | iQ/machine/iQ TiQ8 0.8 nozzle.json | Q chain | Same absent compatibility edge |

R: `Ratrig/process/0.20mm Standard @RatRig.json` →
`Ratrig/process/fdm_process_ratrig_common.json` →
`Ratrig/process/fdm_process_common.json`; all exist. Each RatRig machine inherits
`Ratrig/machine/fdm_klipper_common.json` → `fdm_machine_common.json`; all exist.

Q: `iQ/process/0.20mm Standard @iQ TiQ2 P1 - PACF Pro Fiberthree (0.4 Nozzle).json`
→ `iQ/process/fdm_process_tiq_common.json`; both exist. All six machines inherit
`iQ/machine/fdm_tiq_common.json`, which exists. The internal names use `Nozzle`
while machine filenames use `nozzle`; this case difference is not a missing file.

K1 machine inherits `Creality/machine/fdm_creality_common.json` →
`Creality/machine/fdm_machine_common.json`; all exist. The absent default process
is a missing selection target, not a broken machine ancestry.

The indexed instantiated processes are353 Creality,41 Ratrig and5 iQ; none lists
these target leaves. iQ's four TiQ2 and one TiQ8 process list only the respective
0.4 leaf. The full candidates, literal compatibility lists, exact chains and
original row diagnostics are retained externally, not replaced by this summary.

## Distinguish catalog, harness and CLI behavior

The original stock sweep stopped all10 in test-harness select_printer
(orca_parity.rs:191-273). K1 reports missing default process; the other9 reject
its incompatible default and find no alternative compatible instantiated process.
Neither Native nor public Ares Core was called for those historical rows.
VENDOR_INCOMPLETE therefore was not an observed Ares slicing rejection.

Native system loading and GUI selection are separate boundaries:
PresetBundle.cpp:4898-4921 diagnoses unresolved system inherits; user preset
loading:1509-1525 skips unresolved parents. These ten machine/process chains do
not exhibit that missing-parent condition. Preset.cpp:809-850 checks explicit
compatibility lists; update_compatible_internal:3340-3385 exposes built-in
defaults if none is compatible, and select_preset_by_name:3594-3617 falls back to
a visible/default preset. Those GUI policy paths are not evidence of slicing
these stock selections, and were not invoked in this audit.

Native CLI load_config_file:1953-2033 does not itself select/resolve vendor
parents. However, the subsequent OrcaSlicer.cpp:2556-2654 process_compatible
branch **does** gate loaded machine/process identities, before export or slicing.
Looking only at file parsing or the profile-entry ARD's no-gate wording misses
this phase. No assertion that the complete CLI bypasses compatibility survives
this runtime/source audit.

## Individual CLI diagnostics, unchanged profiles

The nine existing default processes and each machine/default filament were
flattened without changing compatible_printers, placement, options or selectors.
Metadata serialization follows the existing write_preset convention. These are
explicit caller-requested defaults, not a successful stock-selector run or a
repaired compatible selection. All external files/commands stay/tmp.

| Printer | Native exit / result | Ares same argv exit / result |
|---|---|---|
| K1 SE0.8, explicit absent default process path |253 / -3|2 / no result.json|
| RatRig V-Core4 3000.8|239 / -17|2 / no result.json|
| RatRig V-Core4 4000.8|239 / -17|2 / no result.json|
| RatRig V-Core4 5000.8|239 / -17|2 / no result.json|
| TiQ2 0.25|239 / -17|2 / no result.json|
| TiQ2 0.6|239 / -17|2 / no result.json|
| TiQ2 0.8|239 / -17|2 / no result.json|
| TiQ8 0.25|239 / -17|2 / no result.json|
| TiQ8 0.6|239 / -17|2 / no result.json|
| TiQ8 0.8|239 / -17|2 / no result.json|

Native -3 is load_config_file:1955-1957, stdout exit line and missing-setting
stderr diagnostic. Each -17 has empty stderr, stdout `run found error, return
-17, exit...\n`, and exact generic result error `The selected printer is not
compatible with the process preset in the 3mf.` Native's log records
`run 2652: process not compatible with printer.` No3MF or G-code was produced.
A separate machine-only partial-input diagnostic for K1 also returned239/-17;
it is not the absent-default request and does not prove successful partial slicing.

For the coordinator's literal same-argv observation, only argv[0] changed to
Ares; every option/value/path after it remained byte-identical. All ten Ares
processes fail in clap before reading profiles: `unexpected argument
'--load-settings' found`, `Usage: ares <COMMAND>`, exit2, empty stdout, no result.
All stdout/stderr comparisons differ. Missing result is itself a difference;
no wall-clock helper was invoked on nonexistent bytes, no JSON was fabricated
and no Native result was mistaken for Ares output. Coordinator ruling: these
are observations of different CLI product surfaces, **not production REDs**.
Porting the src/OrcaSlicer.cpp argv interface is outside the current libslic3r /
libvgcode rewrite scope and requires a separate user product decision.

Catalog::compose (catalog.rs:176-255) resolves selected profiles/parents but
contains no CLI process_compatible gate. The owning libslic3r predicate is
instead exposed through PresetCatalogHandle::filter_compatible and
filter_compatible_presets (compatible.rs:64-162). Missing selected process there
returns typed InvalidInput naming the process. Public Core was not executed
here: runtime predicate/catalog comparisons are the approved next boundary,
not CLI status/channel equivalence or a new guard in profile slicing.

## Follow-up boundary and evidence

Coordinator rejected extending Ares's CLI argument surface. Retain this argv
observation without a production-fix claim. The follow-up spec compares public
profiles catalog/predicate decisions against freshly acquired Native conclusions
at the libslic3r boundary. Nine -17 rows require runtime evidence of incompatible
process/printer identity; K1's -3 requires evidence of the missing process name.
No new EarlyExit variant or CLI-interface port is needed for this Core API
comparison. The independently pending #30 eight-case slice still awaits
worker4's public early-exit recipe.

Evidence: `/tmp/worker3-vendor10-profile-analysis.json`,
`/tmp/worker3-vendor10-cli-analysis-v2/`,
`/tmp/worker3-vendor10-same-argv-red.json` and matching logs/exit receipts.
Seeded Native SHA59fb88a2…; explicit seed0/one CPU recorded per command.
The first wrapper attempt lacked ORCA_REPLAY_CPU, exited1 before Native and is
preserved separately as infrastructure failure, never relabeled a Native result.
No new test, production code, heavy job or Native product was committed.
