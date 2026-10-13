# Layer-range missing height: Native crash boundary

Decision: retain the existing Ares rejection when range-profile regeneration
requires an absent layer_height. This is not R acceptance and adds no parity
case, production behavior or new guard. Coordinator classified six observed
SIGSEGV outcomes as Native rejection of this input and authorized one existing-
binary gdb capture. Any future retirement needs a separate ruling.

## Observed behavior

Input: an authored 20×20×6 cube with no stored profile, range [1,2] specifying
only sparse_infill_density=40%, and range [3,4] specifying layer_height=0.2.
Its preserved input and identity are outside the repository:
`/tmp/worker6-range-missing-height-e0w4a6ht/{input.3mf,identity.json}`.
The diagnostic input is not included in the R acceptance fixture set.

Official and initialized seeded Native each ran the same input three times,
sequentially on CPU11, each with a fresh directory/cache. All six returned
signal SIGSEGV (-11), emitted no G-code and had no layer vector. No outcome was
retried or counted as parity PASS. This is repeatable crashing rejection,
not a clean validation diagnostic or evidence of defined upstream behavior.
Capture: `/tmp/worker6-range-missing-height-e0w4a6ht/results.json` and the
six per-execution stdout/stderr/default-log directories.

Local official wrapper SHA256:
`1689a2ff46564cf723ad90c663051b58115920c3cce639bd2961340955cebbf3`.
Seeded producer: existing `1k0p5nlbs3zsfdcy08g3y5fb7bcm1qrv` store output;
seed=0, ASLR disabled. No Native binary was rebuilt or altered.

## Single debugger capture and source correlation

One batch gdb invocation on CPU11 followed the existing seeded launcher exec
into its existing `.orca-slicer-wrapped`. Debugger was pinned existing gdb17.2
`fq5hd83bkipznh5n0m3zxcs4gk6hcbjc`, invoked through nix shell. Evidence:
`/tmp/worker6-range-missing-height-e0w4a6ht/gdb-seeded/{commands.gdb,backtrace.log,exit}`.

The stack is:

1. `Slic3r::layer_height_profile_from_ranges`, function offset +399.
2. `Slic3r::PrintObject::update_layer_height_profile`.
3. `Slic3r::Print::validate`.
4. `Slic3r::CLI::run` → `main`.

At the fault, `rax=0`; instruction `mov (%rax),%rcx` tries to read the returned
option's vtable before the following virtual getFloat call. Thus there is no
substituted default at the failing access. The existing binary has function
symbols but no source-line debug information: gdb explicitly says “No line
number information available”. The source location below is a correlation
against checked upstream revision8500fcdcc, not a fabricated DWARF line.

`Slicing.cpp:246-253` loops through each range. At :249,
`it_range->second.option("layer_height")->getFloat()` dereferences the result
without a null check; this precedes interval-width trimming. The fault's
option-return/null-vtable/getFloat sequence matches that expression.
`Format/bbs_3mf.cpp:2914-2935` creates DynamicPrintConfig and deserializes only
options present in the XML, then assigns it into the range map; it does not
insert a missing layer_height here. `PrintObject.cpp:3826-3828` invokes this
range generator when no retained profile survives selection. The observed
null return proves no intervening normalization supplied height for this run.

The C++ null dereference is UB. Its deterministic manifestation in six runs
does not make it a portable height-default rule. R's valid retained profile
path avoids this generator through `PrintObject.cpp:3805-3833`; its explicit-
height path supplies the option. Those are the defined acceptance routes.
Ares keeps its existing missing-height rejection on the regeneration route;
its current None-skip scaffold is not used as Native authority.

## Scope status

R remains governed by [its approved spec](layer-config-ranges-slicing-spec.md).
No public diagnostic API, Native build, new rejection or acceptance behavior
is introduced by this evidence. The premature R RED batch
`/tmp/worker6-range-red.Yunpua` was interrupted during compilation to respect
coordinator's debugger-before-RED ordering and is not evidence. R proceeds
with a fresh public RED after this report.
