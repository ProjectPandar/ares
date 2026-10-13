# printer_technology GREEN addendum: resolved FFF admission

Approved scope after public RED `baa2b41ad`; base audit `96328f269`.
This supplement is committed before any production edit.

## Source boundary and admission rule

`OrcaSlicer.cpp:3511-3512` defaults unknown technology to FFF; `:3619`
writes resolved technology into print config; `:3627-3633` admits only FFF
and otherwise emits `invalid printer_technology ` plus newline and exits
`CLI_INVALID_PRINTER_TECH` (-7, POSIX-visible249).
`PrintConfig.cpp:674-684` defines FFF/SLA with FFF default; schema legality
of SLA is not slicing admission. `DynamicPrintConfig::validate`'s empty
SLA error map remains distinct from this resolved technology admission.

The public RED imported SLA 3MF SHA
`58e96bf776f0b54553372a09a697c601de39606c7818c92aa1584b6424d8787e`
is rejected by both Native modes with249 and the above diagnostic; Ares CLI
currently exits0 and emits G-code. Evidence:
`/tmp/ares-printer-technology-red-baa2-59TjhG/`.

## Minimal Rust boundary

Add a typed `SliceError::InvalidPrinterTechnology` for resolved non-FFF
project config, not `UnsupportedProjectFeature`. Check materialized project
technology in the shared `project/effective_config` validation boundary,
before FFF pipeline execution. Missing technology keeps its existing FFF
default. Reuse typed `ProjectSettings` technology, not ad-hoc string guards.
Public project slicing API and WASM project adapters consume this same
boundary and must return the typed error/string without generating G-code.
Do not change the independent DynamicPrintConfig validation semantics.

Map this variant through `ares-cli/src/failure.rs`'s existing family reporter
to code-7 (249 on POSIX), stderr **`invalid printer_technology \n`** and the
existing Native family result reporting. Do not substitute Rust's legacy
`Error: ...` diagnostic. Keep OS-visible status matching the Native producer.
If the mapping would grow failure.rs past400LOC, split the existing mapping
mechanically first in a separate commit, preserving its statements verbatim.
All changed files remain <=400LOC.

## Verification and exclusions

Before production changes, extend the existing SLA RED to require shared
public API rejection and browser-WASM adapter rejection at the approved
project slicing seam; no post-hoc unit tests. Use the same admitted Native
input, record typed API result, CLI status/diagnostic, and adapter result.
Keep FFF strict Native/API/CLI full-output + raw equality controls unchanged.
API/CLI failure assertions must survive GREEN unchanged.

Keep the original machine-SLA→effectiveFFF historical ApplicationMismatch
FAIL as recorded; its expected-mismatch regression is not SLA admission.
No `verify_value`, comparator, matrix status/classification or SLA slicing
changes. No parser move/precedence repair, unrelated option checks, fallback
or new Unsupported guard. CPU18 outertaskset, jobs1, isolated target/fresh
artifacts; official+seeded serial, START/END and review receipts. Full and
workspace need separate authorization. Coordinator alone merges; no push.
