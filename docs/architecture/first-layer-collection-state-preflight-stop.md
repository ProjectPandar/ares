# First-layer predicate/state preflight: STOP at differing source collections

Status: completed attribution under approved SPEC7975ff214; no production
or test changes. Rebased docs commit49d5590de on main
104d8497097dc7239ad63f7a69f3b418b699b3e8, range-diff1/1 '='.
Mesh candidate98cb4dfbc is already merged in that main via stacked full6.

## Methods and evidence validity

All live inspections used CPU12 outer taskset, fresh Native cache/artifact
roots, existing binaries, ASLR disabled and read-only GDB memory/return
inspection. No inferior function calls, memory edits or source probes.
GDB process exit0 was never used as the Native/Ares outcome: the inferior
exit event, result.json, output existence and SHA were checked separately.

Native has no C++ type DWARF. Disassembly in BOTH producers proves that
Layer::has_extrusions reads the region vector at Layer+0x130/0x138 and
compares perimeter/fill vector headers at region+0xd0/0xd8 and
+0x100/0x108. collect_layers_to_print reads object/support layer vectors
at PrintObject+0xc10/0xc18 and +0xc28/0xc30. Source is Layer.hpp:101/207/295
and GCode.cpp:1718-1778. Region counts below traverse all regions; none is
silently selected. Runtime has_extrusions return values confirm the reads.
Odd-original controls validate nonempty headers and the successful path.

Ares DWARF permits typed reads at extrusion_islands::assign_layer entry,
BEFORE ordered-island wrapping, and schedule::build entry. Nested entities
were read inside the source perimeter/fill collections, so a nonempty
adapter wrapper around an empty source cannot explain this witness.

Ares binary producer9856c12b0 has identical mesh_slicer, schedule.rs,
extrusion_islands.rs and island_print_order.rs source blobs to main104d.
Current main full6 artifacts independently retain the exact odd-rotated raw
G-code SHA and future_gap=true below; this is explicit carry, not a claim
that the older binary was rebuilt on main104d.

## Durable measured outcomes

Input odd-rotated SHA256:
93a493cec3f419398d2cfc04401e0c5f53c373ffe38d2e526c885e876f6b5bc3.
One object,10 object layers, no support layers; Native first slice_z0.1,
print_z0.2. Both Native producers have one region with perimeter entity
count0 and fill entity count0 on their first layer. has_extrusions returns
false; process156/result-100, stderr Errors, no G-code. Both exhibit layer
presence flags [false,false,false,false,false,false,false,false,true,true].

Ares first source layer has one perimeter collection with2 entities and
one fill collection with1 entity, thin-fill count0. These are nested/source
counts, whereas Native counts refer to the region's outer collection:
only emptiness is directly compared here. Its ordered first layer has
island entity counts [2,0], no support. The existing predicate is true and
CLI exits0. Actual first-layer print_z0.2 has4 Inner wall,4 Outer wall and57
Bottom surface XY moves with positive relative E, under M83. Raw API/CLI
G-code SHA, identical across modes and current main full artifacts:
99521ddc83fa146da43b50d88271afb586afc95f2c046f875089130fb6afc6b4.

Odd-original positive control, both Native modes: first region perimeter1,
fill1; runtime has_extrusions=true, support0, all10 layers nonempty,
result0/exit0. Generator-header-only SHA remains
92de92462c3bde6deacb42f3c6e78d4f3512f6963f24905a33ec299820945e31.
Ares source counts [2]/[1], CLI0 and the same raw SHA as odd-rotated.
Odd-reverse remains the known typed-empty-first-layer/no-G-code control;
no additional successful state sampling for it is claimed in this preflight.

## Roots, revisions and preservation

- worker3-first-layer-state-pk96pxob: VOID. GDB run redirection accidentally
  replaced argv, producing Native GUI1/Ares usage2, without slicing. The
  corrected driver explicitly preserves gdb.parameter('args'); not a retry
  of the intended input to seek a PASS.
- worker3-first-layer-state-v2-c3lu2nyu: valid completed seeded odd-rotated
  and Ares odd-rotated collection attribution before full6.
- worker3-first-layer-state-controls-tpe0l4zn: entire batch conservatively
  VOID at full6 freeze. It had exited by termination check, so no PIDs were
  actually signalled; last Ares END16:40:52 was near/after the broadcast.
- worker3-first-layer-state-main104d-9y7stqky: valid fresh rerun AFTER full6
  END: official odd-rotated, both Native odd-original and Ares odd-original.

All roots are under /tmp. Merged full output: /tmp/worker6-stack-104d-full.sDQ4Kd.
Summary receipt: /tmp/worker3-first-layer-state-STOP-main104d.json.
The roots, drivers, read-only snapshots, symbol/disassembly audit and summary
are archived outside Git at:
/home/indexyz/ares-evidence/worker3/first-layer-state-main104d-9y7stqky.tar.gz
Archive SHA256:05d72fbd8d0f9da1fc74a2804056308316e1d0cf4a479eace3c202c8c5bbfc3b.
No Native output is committed. This document's input hashes/counts/outcomes
remain usable if /tmp's10-day cleanup removes supplementary roots.

## Verdict and next boundary

STOP: GCode.cpp:1774-1778 and the existing Rust schedule predicate correctly
consume their different collections for this witness. Do not add another
empty-first-layer check, drop real extrusion paths or imitate a -100 code.
The difference is already upstream of ordered-island/schedule construction.
Collection attribution does not yet locate the first geometry divergence
or prove std::sort is its cause. A separate source-cited geometry/state
localization SPEC follows; sorting/chaining remain unchanged pending review.
