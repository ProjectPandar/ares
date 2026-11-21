# Direct STL plate triangle limit

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`):
`PrintConfig.cpp:12809–12814` declares `mtcpp` as an integer. The action
handler (`OrcaSlicer.cpp:6289,6394–6395`) starts with no limit and reads the
option only when given; the config definition's displayed default is not
applied on omission. Zero explicitly disables the check. The action loop
(`:6297`) follows argument order: a limit after `--slice` does not affect
that slice.

After placement and print-volume classification, the precheck starts a new
count for each plate (`OrcaSlicer.cpp:6559–6580,6610–6647`). It walks that
plate's model objects and instances in order. It excludes skipped instances
and accumulates only instances classified `Inside`; the accumulation itself
does not independently test the authored printable flag. For each included
instance it adds the imported mesh's facet count from each `is_model_part()`
volume (`TriangleMesh.hpp:facets_count`), not from modifiers, negative
volumes, or other volume kinds. Multiple included instances of the same
object each contribute that object's model-part facets. Native stops on the
first running total strictly greater than a nonzero limit; equality passes.

## Rust destination

Accept `--mtcpp` in the option-only CLI's direct STL `--slice 0` and
`--slice 1` routes, with or without loaded presets. Carry the limit to the
existing selected-plate precheck in `ares-core::project_slice::state`, after
its print-volume classification and no-suitable-objects check and before
slicing the mesh. The count is also the existing success report's
`sliced_plates[].triangle_count`. Keep file I/O and process status in
`ares-cli`; do not add this switch to print settings or G-code metadata.

On rejection Native's family is `CLI_TRIANGLE_COUNT_EXCEEDS_LIMIT`
(`Utils.hpp:80`, `OrcaSlicer.cpp:229,6637–6641`): return code `-59` (Linux
process status `197`) and no G-code. Default CLI stderr is empty; the
ordinary exit line on stdout is `run found error, return -59, exit...`.
The Linux report (`OrcaSlicer.cpp:535–586`) has `plate_index: 1`,
`return_code: -59`, `error_string` equal to `Triangle count of single plate
exceeds the limit. Please simplify the model and try to upload again.`,
zero layer height, wall loops, and sparse infill density, an empty
`warnings` array, the caller's `strict_mode`, and no `sliced_plates` entry.
The logged running total and limit belong to Native's optional logging
surface, not the default stderr contract.

Compare process status, stderr, output inventory, Linux report, and whole
G-code with Native using synthetic STL. Two 12-facet objects on one plate
must pass with limits 25, 24, and 0, and fail at 23 although neither object
is over the limit alone. A limit of 23 after `--slice` leaves it unlimited.
Whole-file comparisons use the existing
slicer-banner and report wall-clock normalization.

## Deferred

Saved projects and other model imports, selecting among multiple plates,
object skipping and authored printability flags, input routes that create
multiple instances or model volumes (including modifier volumes), custom
logging controls, repeated actions, and `--mstpp` are outside this slice. No new CLI actions
or policies are added to the `slice` subcommand, the byte API, or WASM.
