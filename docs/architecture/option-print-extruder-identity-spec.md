# print_extruder_id: bounded process-variant identity spec

Status: Proposed.

## Source owner (not physical nozzle selection)

`PrintConfig.cpp:5405-5410` defines `print_extruder_id` as internal `coInts`,
default `{1}`, `nocli`; `:5412-5418` defines the companion
`print_extruder_variant` Strings vector. No schema numeric min/max or
universal active-nozzle vector length is declared. Do not fabricate either.
It is not an alias for `master_extruder_id`, `printer_extruder_id`,
`filament_self_index`, or a G-code tool number.

`DynamicPrintConfig::get_index_for_extruder`, `PrintConfig.cpp:9004-9055`,
selects the first matching variant name and one-based extruder identity:
when the ID vector covers the variant vector, use its entries; otherwise
derive IDs from grouped `extruder_variant_list` token positions. The stride
scales the resulting option index, not the extruder ID. No matching entry
returns -1. `update_values_to_printer_extruders`, `:9634-9683`, dispatches
selection per active physical extruder and has caller-specific miss handling;
the current source all-extruder loop maps a miss to index0. Do not invent a
new Ares fallback or assume all packaged Native binaries implement the same
miss policy without a runtime witness.

`print_options_with_variant`, `PrintConfig.cpp:8344-8373`, contains only the
two ID/variant metadata keys (speed/acceleration entries there are commented
out). `PrintApply.cpp:1166-1169` materializes printer then process variant
views, before filament variant selection. `PresetBundle.cpp:119,3940` is
another composition caller. CLI process replacement/composition sites
`OrcaSlicer.cpp:2997-2998,3098-3102` use the paired keys; GUI `Tab.cpp:7439`
uses the same pair. Direct `--print-extruder-id` injection is not an oracle
for this nocli option.

## Rust destination and scope fence

Existing typed `options/region_fields.rs` stores `OrcaInts([1])`; wire,
registry and config export already retain the key. The actual materialization
is `options/project_variants.rs:32-33`, `index.rs:63-77` process requests,
`printer.rs:209-223` selecting the two metadata vectors. This is an existing
upstream-compatibility shell, not a new Ares nozzle-routing design.

**Proposed implementation is harness/spec only initially**:
`ares-cli/tests/orca_parity/option_coverage/print_extruder_identity.rs`, using
public byte/CLI slicing plus complete Native output/config receipts. No
new core API/helper, production state, or mapping service. Any proven source
parity disagreement stops for a separate production scope.

`master_extruder_id` admission/physical tool selection is out of scope. Freeze
master ID, filament map, physical extruder map, printer ID/variant vectors,
nozzle geometry and actual filament assignment across this slice. Do not
edit master-extruder fixtures, shared routing guards, tool-transition or used-layer
selection. `project_variants/index.rs` is read-only unless runtime RED proves
a precise nonoverlapping process-ID boundary. Variant-miss and load-settings
behavior also stays untouched.

## Included context-bound candidates

Select one already accepted two-physical-extruder stock context with fixed
valid active variants. If no admitted fixture makes process-variant identity
observable, report INCOMPLETE rather than changing master/physical routing
to create an effect. Author changes only to this key and the paired process
variant vector needed to establish a valid source context; preserve original
export and prove every other setting/ZIP payload unchanged.

1. Default singleton `{1}` in a single-extruder context: schema/wire/export
   control; not sufficient for multivariant consumption PASS.
2. Complete map `{1,2}` versus `{2,1}`, with fixed repeated valid variant
   names: demonstrate physical-request ordering in Native's materialized
   process IDs/variants, and match entire same-input Ares output/config.
3. Repeated IDs with complete paired vectors: first-match witness, using an
   explicit finite vector and ordered source identity; no guessed uniqueness
   constraint or silent duplicate elimination.
4. Empty/short ID vector with a fixed complete variant vector/group list:
   source generated-ID branch, distinguish raw preservation from selected
   result. Incomplete IDs are not automatically invalid schema values.
5. Unmatched identity/variant: probe only if Native accepts a safe public
   invocation; record actual rejection or miss materialization for each
   packaged producer. Abort on crash/setup error rather than interpreting
   it as an expected semantic outcome. No fixture repair/retry-to-PASS.

The source facts above motivate these witnesses; they are not runtime
verdicts. Pair each test with exact input, original raw ID/variant vectors,
Native requested/loaded/materialized metadata, complete G-code and actual
Ares equivalent. Whole generator-only output agreement alone does not
establish that the ID selector was consumed: require an independently
observed Native config selection witness. Physical tool moves may remain
unchanged because this upstream set currently contains metadata only;
do not invent a demand that every ID variation must change extrusion.

## Deferred and evidence requirements

Deferred: master admission, physical/printer/filament map semantics,
GUI preset editing, generic process-preset replacement/inheritance, all
unbounded integer/vector permutations, Windows/native installer variation,
new CLI flag for nocli data, and active routing/geometry/G-code algorithm
changes. This is bounded process-variant identity coverage, not exhaustive
coInts classification or a combined rewrite of the four mapping families.

Before code: list failure modes (raw roundtrip mistaken for consumption,
identity confused with zero-based tool index, complete/short vector branch
conflated, first-match changed to last, stride mistaken for identity,
export normalizing authored request away, Native miss policy mismatch,
shared master state altered). Test-first independent real runtime RED before
any production fix; compile/setup failures are not RED. If behavior already
matches, only tests/context-bound classification need change.

Use existing same-input Native runner/caches and public API seams. Save
producer identity/argv/channels/status/config view/input ZIP payload SHA and
whole G-code. Required missing Native is named FAIL; optional absence visibly
SKIP. No golden, parser/comparator relaxation, extra output normalization,
private defensive validation, fallback or new cache. Keep files <=400 LOC.
