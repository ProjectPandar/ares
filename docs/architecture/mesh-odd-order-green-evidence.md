# Odd-order mesh GREEN evidence

Approved boundary: addendum a1bc82ee1c73ed08cbf1010d1447a70f4621ca18.
Public admission RED: 1df965efc7648d5b1debc193ceb1733e8d5ec38c.
Production: 43ffde369156340624fdee0552944e3d38243111, rebased unchanged as
7498165e0f7bd01f2207a9cb4f32bb8eba367581 onto main
6e0349d9f6da4c0dfdae3cedb26ef9cc0a98e412. Range-diff: all six commits `=`.
This document changes no executable source, tests, fixtures or assertions.

## Source port and measured geometry state

Only mesh_slicer/topology.rs changes in production: import the existing
geometry::fixed_gcc_sort_by and compare `left.key() < right.key()`.
TriangleMesh.cpp::EdgeToFace::operator<:531 compares endpoints only;
create_edge_map:535-560 calls std::sort at558. Construction and the already
ported opposite-first pairing at565-613 remain unchanged. The helper and
all downstream interpolation, chaining, extrusion and export code are intact.
The file is137 lines. No new rejection or layer-state override is introduced.

Fresh read-only snapshots from the rebased production binary show complete
39-use sorted sequences and13-face edge-ID matrices matching BOTH pinned
Native producers, for odd-rotated and odd-original. Odd-rotated edge(3,7)
orders its signed local-edge uses as(5,+1),(4,+1),(0,-3): face5/local0 pairs
with face0/local2 as12; face4/local0 is singleton13. Odd-original retains
(7,-3),(11,+1),(12,+1). The fixed helper supplier boundary remains the one
explicitly approved in the addendum; no broader stdlib tie-order claim.

Actual Ares source collections, before the island adapter, now have no
perimeter, fill or thin-fill entities in odd-rotated's first8 layers and
have entities in the last2. Both Native snapshots have the same ten-layer
presence vector; their first Layer::has_extrusions call returns false.
There are no support layers. Odd-original keeps all10 layers populated.
Emptiness is compared across implementations; Native outer-collection counts
are not equated with Ares inner-entity counts.

The existing Rust schedule.rs:219-246 rejection therefore follows the actual
empty geometry. Native GCode.cpp::collect_layers_to_print:1771-1778 checks
the first object/support pair using Layer.hpp:101/207/295. The source error
is "One object has an empty first layer and can't be printed. Please Cut
the bottom or enable supports." Actual Native stderr is `Errors`, from the
aggregate at GCode.cpp:1848-1866/Exception.hpp:44; OrcaSlicer.cpp:6286-6291
records-100, Unix exit156. Ares returns its existing InvalidInput message
"one object has an empty first layer and cannot be printed", CLI1, no G-code.

## Tests and raw controls

On rebased production7498165e0, official and seeded each run16/16 PASS:
13 strict whole-output mesh-edge cases,2 admission cases and1 diagnostic
collector. The collector is not an output-parity PASS. Both rejected inputs,
odd-reverse and odd-rotated, have future_gap=false in both modes.
Existing comparison and every assertion remain unchanged. All13 admitted
API/CLI outputs are byte-identical to each other, with raw SHA unchanged
from original mesh-edge GREEN9856c12b0 in both modes. Selected raw SHA256:

| Input | Ares API and CLI SHA256 |
|---|---|
| cube_control | 4cd286986fdc9f15b9682f17e46237ac8f2c50377ea51dd4fd226b0fb15f64e0 |
| split_original | 122811692fbc8d9e183d799c62bedf6c01749454db575bb33f40f27974b3aaa3 |
| arachne_split | 741aecb49688f97ce582d359c6c848fc4ccae1391497b1ccaaa5f774fbcdbd60 |
| odd_original | 99521ddc83fa146da43b50d88271afb586afc95f2c046f875089130fb6afc6b4 |

Retained mesh_slicer controls:56/56 PASS on43ffde369, including topology,
facet/intersection, dispatch, chaining and slicing mode. Clean rebase carries
that result: the entire mesh_slicer and sorting-helper source boundaries
are byte-identical. This is not a new-main core-all/workspace run.
On7498165e0, workspace all-target/all-feature clippy with-D warnings, fmt
--check and ares-core/ares-wasm wasm32 check with-D warnings all exit0.
The final documentation HEAD receives its own directed/gate receipt.

## Fresh runs, freeze correction and durable evidence

All outer cargo/nextest/Native/GDB runs use CPU12, directed jobs1/threads2,
fresh Native cache/artifact roots, START/END inbox. Official wrapper SHA256:
1689a2ff46564cf723ad90c663051b58115920c3cce639bd2961340955cebbf3.
During FULL7, driver/GDB tree PIDs680253/680526 were killed and the entire
partial sort root zugzqqtj marked VOID. No paused process was resumed.
Post-END complete snapshots use fresh roots, and rebase snapshots use
another fresh set. The VOID root is retained but excluded from GREEN.

Rebased directed/gates: /tmp/worker3-odd-order-green-6e0349-48dh0fx7.
Rebased sort/pair: /tmp/worker3-odd-order-green-sort-p1rtsj4b.
Rebased collections: /tmp/worker3-odd-order-green-collections-vc4lj6nu.
Core controls: /tmp/worker3-odd-order-green-9vxp1f3t/core-controls.json.
These roots contain complete commands, environments, input/binary hashes,
raw artifacts and machine-readable agreement audits. No Native output is
committed. An archive outside Git preserves those summaries and raw evidence
against /tmp cleanup, including the separately classified VOID root:

/home/indexyz/ares-evidence/worker3/odd-order-green-7498165e0.tar.gz,
3217964 bytes; SHA256
abc22723f4e929fbdc69d432bc0f4f135a7db901d63721a49015c2f0530093ec.

No residual was observed. Workspace/full and merging remain separately
allocated by the coordinator.
