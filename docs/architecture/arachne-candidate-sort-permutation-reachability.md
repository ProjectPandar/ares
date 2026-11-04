# Arachne available-candidate std::sort permutation: reachability and port spec

Status: proposed. No production implementation is part of this document.
Baseline main `3c1a7994164082964407c7a0179694c5c25cb5b8`.
Application source pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
The previously deferred >16 permutation is now investigated separately from
[the open-before-closed fix](arachne-open-wall-candidate-order.md).

**Cardinality is REACHABLE**, including >16 open candidates. **Output RED is
NOT FOUND in this finite nine-input domain.** These are separate conclusions;
neither a universal output-equivalence proof nor UNREACHABLE is claimed.
Do not implement the proposed port until an attributed real Native complete-byte
RED exists.

## Owning upstream boundary and current Rust destination

`PerimeterGenerator::process_arachne` owns this operation:

- `PerimeterGenerator.cpp:2126–2128` processes each surface/island separately.
- `:2287–2293` flattens that surface's generated inset paths.
- `:2295–2307` applies `WallToolPaths::getRegionOrder` constraints.
- `:2319–2324` enumerates unprocessed/unblocked candidates in index order.
- **`:2326–2328` uses std::sort, comparing only is_closed ascending.**
- `:2333–2349` chooses the first eligible strict minimum double distance;
  `:2352–2363` appends the winner, unblocks successors and updates position.

`Arachne/WallToolPaths.cpp:871–872` skips equal inset indices when building
constraints; adjacent-inset/odd rules at873–887 can limit availability.
Therefore disconnected objects/islands do not aggregate into one sort range,
and many generated walls do not automatically imply many available candidates.

Rust destination is the existing temporary upstream-validation shell
`crates/ares-core/src/project_slice/perimeters/arachne/order.rs::order_walls`.
Its current `available.sort_by_key(is_closed)` preserves same-key order.
`materialize.rs:143–171` calls it once per generated surface. This proposal is
an upstream operation rewrite, not a new Ares pipeline or path optimizer.

## Executed public Native whole-output probes

Two batches used public project bytes→Native output and Ares complete output
via `OrcaRunner::rerun_reference` / `compare_case`, with unchanged generated-by
normalization. All timing, M73, coordinates, role and config lines remain.

| Input | Geometric estimate, not count proof | Actual full official verdict |
| --- | --- | --- |
| holes-3x5 |15 holes,16 closed boundaries|PASS|
| holes-4x4 |16 holes,17 closed boundaries|PASS|
| holes-3x6 |18 holes,19 closed boundaries|PASS|
| holes-4x5 |20 holes,21 closed boundaries|PASS|
| holes-5x5 |25 holes,26 closed boundaries|PASS|
| holes-5x5-one-hole-shift01 |one hole moved0.1mm, same25-hole topology|PASS|
| neck-chain-18 |18lobes/17narrow-neck links|PASS|
| neck-ring-20 |20lobes/20narrow-neck links|PASS|
| comb-20 |20thin teeth on one connected backbone|PASS|

Both batches completed all Native calls; no timeout/error/UB verdict occurs.
Each batch used one temporary discovery test.
Six plus three comparison outcomes are individually recorded, not inferred
from an empty/suppressed failure list. Input manifests retain NOT_RUN as their
original preparation state; actual outcomes are in separate probe-records.

Fixtures extend the frozen nearest narrow-neck input using ordinary single
watertight meshes, one object/volume/region, Arachne, one wall, three0.2mm
layers (height0.600000024), no fuzzy/support/infill/scarf. The candidate plan
initially suggested0.4mm height; actual probes preserve the validated template's
0.6mm mesh/component placement to avoid changing import transforms.
`gcode_label_objects=0` is an explicit ordinary input option chosen before
running any probe, to isolate sorting from known official object-id UB; no
output labels or any other G-code bytes are filtered.

The first harness attempt stopped before Native because the assertion used
fuzzy enum `none` while the input's actual spelling is `disabled_fuzzy`.
The corrected assertion verifies that exact existing value; the first run is
preserved as a precondition
error, not RED. A fresh independent cache/artifact root was used afterward.

## Existing-binary observations establish cardinality, not output RED

GDB was invoked through
`nix shell nixpkgs#gdb -c gdb`, against the **existing official binary**;
Orca was not rebuilt, patched or memory-mutated.
The ELF retains `process_arachne` and its dedicated introsort symbols but not
source debug lines. Read-only register/memory interpretation is grounded in
retained disassembly and `Arachne/utils/ExtrusionLine.hpp` field layout:

- introsort entry at ELF0x21cdf30: rdi/rsi are candidate iterators; rcx points
  to all_extrusions vector; return-address filtering excludes recursive calls.
- comparator assembly reads is_closed at line-pointer+9, matching the bool
  after inset_idx/is_odd. Junction vector begins at+16; coord_t is int64.
- process ELF0x21c7131 is after final insertion and before greedy scan;
  candidate range is rbx/r14, vector data at rsp+0x1e0, current Point at+0xa0.
- process ELF0x21c6d1d exposes the actual winner in r9. Every observation's
  source-based diagnostic selection matched this observed winner.

| Existing Native input | Observed selection rounds | Maximum available | Maximum open | Maximum closed | Rounds total>16 | Winner differs from stable-key counterfactual |
| --- | --- | --- | --- | --- | --- | --- |
| holes-5x5 |78|26|0|26|30|0|
| neck-chain-18 |105|35|17|18|57|0|
| neck-ring-20 |120|40|20|20|72|0|
| comb-20 |63|21|20|1|15|0|

The25-hole case's30 large-range rounds all reorder equal-key indices, but
none has a minimum-distance tie. The open structures contain some open/closed
shared-front ties; these do not change the winner relative to stable boolean
sorting. No large-range same-key minimum tie was observed in these samples.

All four debugger-produced Native complete outputs equal their corresponding
fresh Native outputs under generated-by normalization only. Raw SHA differences
are retained rather than called equal. This rules out an unobserved debugger
output change for these four replays, not general Native determinism.

Evidence: a GDB summary, four per-case GDB records, the official run outputs,
two probe records,
input manifests and Native producer identities. Nine input+raw-output hashes
join exactly to fresh Native outputs. The frozen index and SHA are below.
Temporary Rust harness/registration were removed after discovery.
No production Rust changed.

## Proposed deterministic port, conditional on real output RED

The call site remains `PerimeterGenerator.cpp:2326–2328`. Its implementation
supplier must also be pinned: [GCC14.2.0 libstdc++ stl_algo.h](https://github.com/gcc-mirror/gcc/blob/releases/gcc-14.2.0/libstdc%2B%2B-v3/include/bits/stl_algo.h).
The official ELF comment identifies Ubuntu GCC14.2.0/clang18.1.3/LLD18.1.3;
this alone is not full build/header provenance. Retained Native disassembly
confirms threshold16 and the observed compare/partition operation. Do not
substitute the PATH's GCC16 or the initialized reference's GCC15 identity.

Proposed Rust boundary: a private sibling module, e.g.
`perimeters/arachne/candidate_sort.rs`, taking the existing mutable candidate
index slice and immutable extrusion slice. Only replace the sorting call;
retain construction, dependencies, strict norm comparison, empty-path handling,
winner/current updates and traversal unchanged. No generic public sorting API,
FFI, platform-native library calls, feature switches or sampled permutations.

Included upstream algorithm behavior:

- `__move_median_to_first`/`__unguarded_partition_pivot`: same comparator
  branches and median swap, including equal-key choices.
- `__unguarded_partition`: identical bidirectional advancement and swaps.
- `__introsort_loop`: threshold16, depth2*floor(log2(n)), right recursion
  followed by left iteration; preserve the specified operation order.
- `__final_insertion_sort`, guarded/unguarded insertion: same stable small-set
  behavior and shifts after introsort; Rust sort_unstable is not equivalent.
- depth exhaustion: upstream heap selection/sort operation, with its equal-key
  child/sift choices, from [stl_heap.h](https://github.com/gcc-mirror/gcc/blob/releases/gcc-14.2.0/libstdc%2B%2B-v3/include/bits/stl_heap.h).
  Do not silently choose a different algorithm for that branch.

Implementation would mutate only existing indexes in place, with the upstream
logarithmic recursion budget and no new heap allocation for sorting. Its
boolean comparator is exactly the existing is_closed key, not a secondary
index tie-break. Platform-neutral Rust would deliberately implement this
specified reference operation on Linux/Windows/macOS/WASM; no claim is made
that every C++ standard-library implementation shares this equal-key order.

Deferred: other std::sort sites, general comparator framework, other stdlibs'
permutations, polygon/path generation/stitching/region constraints, seam B,
visibility, float/distance arithmetic, thin-wall graph changes and Native UB.
Existing order/materialize scaffolding remains the temporary shell for the
cited upstream operation; this slice does not redesign any slicing stage.

## RED gate and next finite investigation

**No implementation GREEN may follow this report's nine PASS cases.** Next
search must target a same-key minimum-distance tie while Native availability
still exceeds16, rather than merely add more independent islands/holes.
A connected symmetric junction with many remaining open branches is a lead;
public coordinates can be designed/perturbed, but Native path fronts must be
observed rather than replaced with fabricated geometry outputs.

A valid future RED must freeze actual Native/Ares whole outputs and first
byte/line, retain any earlier M73 difference, and attribute the differing
winner to the recorded sort permutation (rather than just source suspicion).
Commit RED alone before any production implementation. Then the
port requires whole-output GREEN and fresh paired full seeded and
official controls, all state/reason changes, workspace/clippy/fmt/wasm checks.
If the finite continuation still has no RED, report NO RED FOUND with its
exact domain; do not label cardinality UNREACHABLE or implement without RED.

## Frozen evidence

Evidence index: nine matched Native cases, 196 files;
SHA256 `03306e26f000809ff8b93a09541835fa601ad7442fc9e8591634476ac06a3268`. Official executable SHA256:
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.
Cache metadata retains actual native/resources/launcher/process identity.
Full workspace/seeded/clippy/wasm gates
are not claimed for this docs-only result; last production remains the
open-before-closed slice. Final `cargo fmt --all -- --check` PASS;
final staged diff check PASS. No production Rust or lasting test changes.
