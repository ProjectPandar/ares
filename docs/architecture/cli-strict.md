# Native CLI strict warning policy

OrcaSlicer v2.5.0 `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

## Source boundary

- `src/libslic3r/PrintConfig.cpp:12824–12837` defines the two booleans.
  `src/OrcaSlicer.cpp:1388–1397` records strict mode before loading inputs;
  pairing it with `--no-check` prints `--strict cannot be combined with --no-check`
  to stderr, reports `CLI_INVALID_PARAMS` (−2, POSIX 254) at plate 0, and exits.
- The CLI scans `g_slicing_warnings`, not validator messages, logs, or G-code
  processor warnings. Both callbacks append `PrintBase::SlicingStatus` with an
  actual warning step (`OrcaSlicer.cpp:453–472,7057–7088`). `PrintStateBase`
  stores severity and current step warnings (`PrintBase.hpp:61–106,299–318`),
  and `PrintObjectBase::active_step_add_warning` forwards status through
  `PrintBase` (`PrintBase.hpp:437–480,678–682`). `PrintObject::generate_support_material`
  issues the only reachable noncritical notification, `SlicingNeedSupportOn`,
  when `is_support_necessary()` finds an unsupported feature with support off
  (`PrintObject.cpp:966–986,5289–5303`).
- `OrcaSlicer.cpp:7141–7161` scans nondefault statuses after `Print::process`
  and conflict handling, before any G-code or statistics export. It saves the
  warning text in the plate report and records `slicing_warning_non_critical`.
  Strict mode stops at the first such warning and returns `CLI_SLICING_ERROR`
  (−100, POSIX 156; codes `Utils.hpp:44,92`). The plate entry is pushed before
  export populates triangle count and times, so those fields are still zero.
  This noncritical action does not print a stderr diagnostic at the default
  log level. `record_exit_reson` writes `strict_mode` and the accumulated
  `warnings` in the Linux report (`OrcaSlicer.cpp:535–574`).

## Rust destination and limits

The Native-style Ares `--slice … --outputdir …` routes accept `--strict`.
Reuse the already ported `SlicedPlate.warnings` snapshot of typed
`PrintWarning`s, and the CLI's result writer. Check the noncritical step
warning after slicing and before writing either G-code or statistics; do not
recompute support geometry or treat config substitutions as strict failures.
Set `strict_mode` from the request on both success and failure; with the flag
absent the respective existing outputs remain unchanged. Reject the conflicting
pair before loading input, independently of argv order. The CLI core may have
computed in-memory G-code by the time the adapter checks the warning; no
filesystem output may have started. Standalone `--no-check` check suppression,
other actions and the Ares-only `slice` subcommand are not included.

## Public witness

Reuse the synthetic two-object cantilever 3MF in
`tests/parity/spiral-vase-admission/two-object-by-object-apart.3mf`.
With support disabled, Native selects the floating cantilever warning for
`overhang.obj`. Without the flag: exit 0, empty stderr, `plate_1.gcode` and
`result.json`, report carries the warning and `strict_mode: false`. With
`--strict`: exit 156, empty stderr, only `result.json`, first warning and plate
entry retained with `strict_mode: true`. With `--strict --no-check`: exit 254,
exact conflict stderr, only `result.json` with no sliced plate and no warnings.
Compare exit, stderr, sorted output names and the full report after the usual
wall-clock normalization against Native in both reference modes.
