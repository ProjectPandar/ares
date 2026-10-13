# Arachne fuzzy F2 integration audit

This implements the accepted 791a9fbe9 dependency ruling on top of the
single-group kernels, after multi-group Native RED fc5d5051a (rebased
bb985c4c8). F1 and F2 remain separate production commits and are reviewed,
validated by canonical full and merged only as one complete stack.

## Source and Rust boundaries

- FuzzySkin.cpp:415-473 `group_region_by_fuzzify`: actual compatible
  `PerimeterGroup.members` are exposed by types.rs; fuzzy.rs groups full
  quantized configurations, computes contour/hole flags and constructs
  default Miter3 safety-offset masks. Layer compatibility is unchanged.
- PerimeterGenerator.cpp:2094/2119-2122/386-387/422-428: materialize.rs
  prepares the group before wall generation, suppresses arc-fitting's
  resolution reduction for active contours, passes planned slice_z and
  the execution-local SlicerContext; traverse.rs applies junction noise
  before conversion and uses the upstream fuzzy reversal flags.
- Layer.cpp `make_perimeters`: materialize.rs visits physical-layer
  compatible groups in discovery order, keeping RNG consumption separate
  from existing region-major result storage.
- FuzzySkin.cpp:502-558/696-814: fuzzy.rs filters eligibility, merges equal
  effects, unions masks, subtracts later masks, splits each current line,
  inherits source width/index, and applies upstream segment isolation and
  final endpoint assignment. The latter belongs only to this multi-region
  branch; no closing repair is introduced in the whole-line Ripple kernel.
- Algorithm/LineSplit.cpp:19-28/64-314: geometry/line_split.rs uses indexed
  subject junctions, MAX clip tags and the negative source-edge callback;
  orders clipped sections before reconstruction. It never uses the existing
  overhang width-interpolation callback for fuzzy crossings.
- AABBTreeLines.hpp:222-237 and AABBTreeIndirect.hpp:635-663: the existing
  LineDistanceTree supplies a lazy tree with strict-radius, left/right
  traversal for clip-vertex source resolution, including upstream's explicit
  first-nearby-edge fallback.

## Preservation and validation status

The obsolete rejection unit was retired separately under explicit approval.
The remaining 27 traverse unit assertions are unchanged; only context setup
and call arguments were adapted. The existing public fixtures, comparison
assertions and full ordered-byte comparator remain untouched. Classic fuzzy
sampling is unchanged; its existing coherent-noise implementation is reused.

The first workspace strict compilation passed. Final strict/fmt/WASM checks
are recorded in external engineering artifacts. Public F1/F2 Native directed
validation remains pending the coordinator's W2 END broadcast; no GREEN claim
or parity PASS is made before those checks. No full/workspace run or push is
started by this implementation step.

## Overlap-priority public GREEN defect and consumer fix

The unchanged overlap-priority fixture failed strictly in both modes on
016645d74. Canonical wrapper SHA was 1689a2ff46564cf723ad90c663051b58115920c3cce639bd2961340955cebbf3.
Debugger attribution, not parity evidence, is stored externally under
`/tmp/worker3-fuzzy-overlap-boundary-debug-1jp457o3`: all eight inner/outer
lines have identical Native/Ares input and output junctions at
PerimeterGenerator.cpp:386-387 / FuzzySkin.cpp:685-814. Coordinates, widths
and perimeter indices match, confirming masks, priority and ordered splits.

`/tmp/worker3-fuzzy-overhang-entry-nnjosyk5` confirms Ares received no
previous boundary on any of the four physical layers although overhang
speed was enabled. EntryBuilder passes a physical layer index, but the old
traversal.lower_slices helper used it as a region-major record index.
The two modifiers emptied the base region, so those slots were None.

GCode.cpp:4853-4860 and ExtrusionProcessor.hpp:305-311
`prepare_for_new_layer` use the previous physical layer's merged
Layer::lslices. The traversal helper now resolves that physical layer via
its existing slices view (which searches actual nonempty region records),
instead of depending on a base region. GCode.cpp:6657-6718 then reaches the
existing overhang segmentation consumer. This required F2 consumer repair
introduces no new fuzzy kernel, mask rule, normalization or guard. Keep all
37 public cases, comparison assertions and fixture bytes unchanged.
