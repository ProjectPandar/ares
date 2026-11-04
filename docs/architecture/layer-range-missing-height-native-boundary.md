# Layer-range missing height: Native crash boundary

Decision: retain the existing Ares rejection when range-profile regeneration
requires an absent layer_height. This is not layer-range acceptance and adds no parity
case, production behavior or new guard. The six observed SIGSEGV outcomes are
classified as Native rejection of this input. Any future retirement needs a
separate decision.

## Observed behavior

Input: an authored 20×20×6 cube with no stored profile, range [1,2] specifying
only sparse_infill_density=40%, and range [3,4] specifying layer_height=0.2.
The diagnostic input is not included in the layer-range acceptance fixture set.

Official and initialized seeded Native each ran the same input three times,
sequentially, each with a fresh directory/cache. All six returned
signal SIGSEGV (-11), emitted no G-code and had no layer vector. No outcome was
retried or counted as parity PASS. This is repeatable crashing rejection,
not a clean validation diagnostic or evidence of defined upstream behavior.
The seeded producer ran with seed=0 and ASLR disabled. No Native binary was
rebuilt or altered.

## Single debugger capture and source correlation

One batch gdb invocation followed the existing seeded launcher exec
into its existing `.orca-slicer-wrapped`.

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
does not make it a portable height-default rule. The valid retained profile
path avoids this generator through `PrintObject.cpp:3805-3833`; its explicit-
height path supplies the option. Those are the defined acceptance routes.
Ares keeps its existing missing-height rejection on the regeneration route;
its current None-skip scaffold is not used as Native authority.

## Scope

Layer ranges remain governed by [their spec](layer-config-ranges-slicing-spec.md).
No public diagnostic API, Native build, new rejection or acceptance behavior
is introduced by this analysis.
