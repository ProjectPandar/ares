# Printer technology: native oracle boundary

Status: classified as reference unavailable (`ORCA_ERROR`);
SLA is excluded from further execution, not counted as a parity PASS.
No Rust implementation change.
This is an investigation record, not a proposed Ares slicing pipeline milestone.

## Reproduction at16ac907b

Reuse the frozen workspace archive from the verified16ac907b build, set
`ARES_PARITY_OPTIONS=1 ARES_OPTION_KEYS=printer_technology`, and run
`option_coverage`.
Official and seeded references run serially and both exit100:
2 values generated,1 strictly compared,0 rejected; SLA requests export FFF.
The bounded domain remains FAIL; this is not evidence of complete coverage.

The retained input machine JSON explicitly contains `printer_technology=SLA`.
The process and filament JSON do not contain the key. This rules out wrong
preset injection and an explicit sibling-preset FFF override.

A one-variable experiment adds `--printer-technology SLA` to the same export
command with unchanged preset files/model. Both official and seeded binaries
exit249 (native -7), printing `invalid printer_technology`. Changing only that
argument to FFF succeeds with exit0 on both.

## Source explanation

- `libslic3r/PrintConfig.cpp:678` defines the technology enum; its existence
  does not prove the Orca CLI executes both technologies.
- `OrcaSlicer.cpp:2070,2096` moves each loaded machine/process `config` into
  its destination before `get_printer_technology(config)` at2100.
- `libslic3r/PrintConfig.hpp:620` delegates move assignment to
  `DynamicConfig`; `Config.hpp:2786–2792` moves and explicitly clears the
  source options. The subsequent technology lookup sees no key.
- `OrcaSlicer.cpp:3510–3512` defaults unknown technology to FFF and3619
  writes the selected technology into the final print configuration.
- `OrcaSlicer.cpp:3622–3644` initializes only `FullPrintConfig` for FFF;
  a non-FFF technology exits with `CLI_INVALID_PRINTER_TECH`. SLA config
  initialization is commented out. The explicit-argument experiment matches
  this rejection, unlike the preset-only path that silently exports FFF.

## Decision and deferred work

Record SLA as reference-side `ORCA_ERROR`
(native -7) and skip further SLA execution. Do not implement SLA. This explicit,
evidenced exclusion supersedes the earlier unresolved application-failure
classification; keep the original raw reports unchanged. FFF's successful
comparison remains valid, but the two-value domain is not fully compared.
Do not whitelist SLA as equivalent to FFF or classify it as a passing comparison.
No unsupported guard is added to Ares; the failure occurs before Ares slicing
and is not in `gcode_emit/cooling`.
Do not patch the official reference or invent an Ares SLA pipeline to make a
report green. A future SLA rewrite would require a separate,
source-cited libslic3r boundary and a reference/output contract that actually
supports it. Keep this native-oracle limitation explicit while pursuing other
option groups. No comparator, activation or production Rust file was changed.
