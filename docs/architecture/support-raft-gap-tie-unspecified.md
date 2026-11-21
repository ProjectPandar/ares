# Do not port equal-Z raft gap layer selection

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) can schedule a
Slim tree raft gap and the first tree base at the same print Z for one
object. Which record survives in the exported G-code is not specified.
Keep Ares scheduling unchanged rather than reproducing the reference
build's tie order.

## Upstream boundary

- `src/libslic3r/Support/TreeSupport.cpp:1362–1376,1643–1652` creates and
  fills the raft gap; `:3537–3543,3591–3598` starts the tree support layer
  plan at the first object layer's print Z. The two passes can place
  distinct support layers at that Z.
- `src/libslic3r/GCode.cpp:2353–2384` builds per-object records.
  `:2528` sorts their ordering entries with `std::sort` using only `print_z`;
  `:2532–2547` merges nearby Z values into a chunk with one slot per
  object. Equal-Z records have no tie-breaker. A second write to an
  occupied slot violates the assertion at `:2545`. With assertions
  disabled, `:2546` overwrites the entire record, including any paired
  object layer. The winner depends on the order produced by the sort
  implementation, not on an upstream raft or support precedence rule.

For example, with `raft_layers=1` at Z=0.5 mm, the reference executable
first wrote the 0.3 mm gap layer (support ID 1, paired with the first
object layer) and then overwrote it with the 0.2 mm tree base (support
ID 2). It emitted the tree base. The gap's input polygons, infill paths,
and empty perimeter output agreed with Ares before the merge. This is a
layer selection difference, not an offset, fill, or arc-fitting discrepancy.

## Fixture selection

For strict ordered G-code comparisons, avoid fixtures where Slim with
`raft_layers=1` puts the raft gap and the first tree base of one object
in the same export Z chunk (`EPSILON` at `GCode.cpp:2536–2537`). Exclude
those fixtures at selection time, not by masking output or changing the
comparator. A non-colliding raft control remains valid. This is not a
blanket exclusion of Slim or of single-layer rafts, and it does not
change which projects Ares can slice.
