# Slice 3: account finite print_extruder_id witnesses in the option matrix

Documentation only, no RED or implementation. Audit baseline 96328f269 on
executed f637b4122; preserve the exact historical sweep evidence.

## Upstream rewrite boundary

- libslic3r/PrintConfig.cpp::PrintConfigDef::init_fff_params:5405-5417 defines
  print_extruder_id coInts and its paired print_extruder_variant coStrings.
- DynamicPrintConfig::get_index_for_extruder:9004-9055 resolves variant
  matches using complete explicit IDs or generated extruder-group identity.
- DynamicPrintConfig::update_values_to_printer_extruders:9634-9683 selects
  the per-physical-extruder variant slots, including the existing upstream
  no-match handling; this slice introduces no numeric/index policy.
- PrintApply.cpp::Print::apply:1165-1172 materializes printer then print
  variants before filament variants. GCode.cpp:5591-5644 append_full_config
  independently emits the materialized print configuration.

This is accounting for existing source-cited rewrite witnesses, not a new
Ares identity subsystem. Existing loaders are temporary compatibility shells
for these upstream configuration/materialization boundaries.

## Rust destination and included finite domain

Reuse crates/ares-cli/tests/orca_parity/option_coverage/
print_extruder_identity.rs and its six existing tests, fixtures, Native
materialization assertions and complete-output comparator without weakening
or replacing any assertion. Add the narrowly scoped generator/executor hook
in domains.rs and sweep.rs; option_coverage.rs only if module wiring requires
it. Extract the six case descriptions into a shared local table so the matrix
and individual tests execute the same requests and expectations. Split a
local input/case file if necessary to keep every file at most400 lines.

| Case | Printer | Requested IDs | Expected materialized IDs |
|---|---|---|---|
| reversed | H2D | [2,1] | [1,2] |
| ordered | H2D | [1,2] | [1,2] |
| default | X1C | [1] | [1] |
| repeated | H2D | [1,1,2] | [1,2] |
| short | H2D | [7] | [7,7] |
| unmatched | H2D | [1,2] | [1,1] |

The exact existing variant vectors and expected materialized variants remain
part of each case, especially repeated/short/unmatched. These are six
Native-admitted contexts, not six arbitrary legal ID vectors. Preserve ZIP
member equality outside the two requested settings, frozen master/physical
maps, original/exported/authored input and requested/effective proof. Reuse
existing retained Native/API streams and comparison, not snapshot goldens.
Do not route these vectors through the generic Ender preset injector.

Replace this key's None-valued matrix baseline with six explicit requests,
record paired variant/context/source/deferred metadata in the plan, and
credit a case only after its existing full comparison and Native materialized
view assertions pass. A selected-value subset cannot certify the full scope.
No credits from an absent runner or an unexecuted individual test.

## RED before GREEN

First commit a public matrix integration regression: request only
print_extruder_id via the existing option sweep and assert its generated plan
contains all six explicit requests with their contexts; require the external
matrix outcome to account six complete comparisons and the six case receipts.
The current one-baseline INCOMPLETE row must fail the accounting expectation.
Run existing six tests as positive Native controls in official and seeded.
This is a harness/accounting RED, not an alleged new slicer behavioral RED.
Commit the RED before changing the generator/executor.

GREEN connects the existing witnesses to the plan; keep all six standalone
regressions and assertions. Stop on any Native rejection, changed materialized
view, missing receipts or behavioral difference; do not repair production in
this slice. Fresh root/cache/target per mode. Gate fmt/strict
clippy/wasm32 after GREEN.

## Deferred and acceptance

Defer arbitrary vector shapes/ranges, other variants/printers, malformed or
invalid-ID policy, master selector, physical/printer identity, ToolOrdering,
external API changes and production fixes. Native's short/unmatched behavior
is retained exactly, not generalized into an Ares fallback.

Exit: six plan cases and six witnessed comparisons in each mode, explicit
bounded scope/deferred metadata, no change to other domains, same existing
full-output equality.
The historical audit remains unchanged; later evidence reports a new hash.
