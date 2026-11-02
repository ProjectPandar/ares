# Parity CLI supply: preserve explicit request modes

## Boundary and dependency

This slice corrects the test-adapter request boundary in
`crates/ares-cli/tests/orca_parity/runner/application/cli_command.rs`.
It does not add a slicing pipeline or change Native/Ares production CLI policy.
The owning source distinction is `OrcaSlicer.cpp`'s explicit loaded-settings and
loaded-filaments paths versus optional project-system refresh at `:2774–2816`:
explicit loaded filaments replace their slots and are skipped by optional refresh
(`:2796–2798`). The approved project-derived adapter contract likewise makes
`--project-preset-catalog` mutually exclusive with explicit replacement/resource
supply and requires that catalog for the explicit Ares `--uptodate` control.

The bugs occur only with the held refresh adapter/CLI dependency. Main
`5910df8eb` has neither that adapter nor those catalog/up-to-date flags, so a main
unknown-argument failure would not be a qualified RED. This separate branch starts
from the approved **logical** held stack, without any mechanical caller patch;
approved branches stay unchanged. This dependency exception was explicitly chosen
and approved after reviewing the missing main seam.

## Problem

`CliCommand::output` currently adds the runtime-supplied catalog whenever argv does
not already contain `--project-preset-catalog`. This alters two real public request
protocols:

1. An authored filament-only `--load-filaments` replacement becomes an invalid
   combination of mutually exclusive modes. The CLI correctly rejects the invalid
   request the adapter manufactured.
2. Authored `--uptodate` without its required explicit catalog is intentionally
   invalid. Automatic injection repairs the negative request and incorrectly makes
   it succeed, defeating the original protocol assertion.

## Decision: classify argv mode once, before launching

Uniformly recognize authored explicit mode flags: `--project-preset-catalog`,
`--load-settings`, `--load-filaments`, `--resources-dir`, and `--uptodate`.
Recognize accepted long-option assignment spelling as well as a separate value.
When any is present, keep the command argv unchanged. Otherwise ordinary project
requests receive the runtime catalog through the shared adapter, just as before.

This is a request-mode boundary, **not** a fixture/test/preset allowlist. The same
rule applies to every caller with that argv shape. No error-dependent retry,
second bare invocation, environment unsetting, catalog artifact edit, Native
precondition change, or assertion/comparator relaxation is permitted.

Explicit catalog requests already carry their own bytes and remain unchanged.
Explicit replacement/resource requests keep their existing semantics and errors.
Explicit up-to-date remains a protocol control: a missing required catalog stays
missing and is rejected by the public CLI, not secretly repaired by parity supply.
Malformed/missing explicit arguments remain the CLI's own errors; the adapter does
not validate or synthesize replacements for them.

Core, production CLI flag definitions, catalog producer, Native argument supply,
and preset refresh application are unchanged. No new warning/logging or generic
configuration layer is introduced.

## Test-first qualification

Add tests that invoke the existing shared CLI adapter directly; no mass constructor
rewiring is necessary for this slice's RED.

- Reuse the exact existing filament-only fixture/filament JSON and Native command
  from `load_settings_3mf_partial::load_settings_3mf_filament_only_whole_file`.
  Native runs first, exits0, and produces complete output. The authored Ares argv
  is only `slice --load-filaments ... -o ... project.3mf`. With runtime catalog
  present, baseline rejection of the injected conflicting mode is the RED. Require
  the original complete generator-normalized comparison, not merely admission.
- Invoke the real public `slice --uptodate ... project.3mf` request through that
  adapter. Baseline succeeds because the adapter supplies the missing flag; the
  protocol requires exit2 and the missing required catalog diagnostic. No Native
  equivalent is invented for this Ares adapter-only required-argument contract.
- Add an ordinary project positive control using the same adapter, with no explicit
  mode flag, against staged Native's strict applied CONFIG_BLOCK. It must retain
  automatic catalog supply after the correction. An absent environment key runs
  the genuine bare request; it is not a skip or an automatic pass.

Before implementation enumerate mode failures: conflicting explicit replacement,
negative up-to-date repaired, explicit catalog duplicated, ordinary supply dropped,
long-option assignment overlooked, and failed request retried as bare. Use existing
CLI missing-file/mutual-exclusion controls together with the new direct controls;
write all assertions before logic changes. Existing two failing original tests,
fixture bytes, ROOT registration and comparison bodies remain untouched.

## Scope and gates

Included: argv-mode-preserving shared parity adapter, the two real public protocol
regressions, and ordinary/explicit request controls. Deferred: production CLI mode
changes, C1 preflight integration, additional slicing/configuration behavior, and
whole-inventory acceptance. The separate C1 branch is queued independently; do not
rebase it into this slice.

Commit ARD and qualified RED, then request review **before GREEN**. After approval,
make the minimal shared predicate change, run both modes and explicit/ordinary
controls, regenerate mechanical wiring only on an accepted combined verification
tree, and re-run the unchanged original two IDs. Refresh remains HELD until the
complete P→F gate is met; this harness correction grants no failure exemption.
