# ZAA stage1: deterministic ray kernel and perimeter contour checkpoint

SPEC06a8 / RED6d2b6f37a / repair PLAN90381524d. The staged condition permits
contour now and requires a separate mechanical fill migration.
This checkpoint is not whole-feature GREEN. The capability
rejection remains; FillExtrusionPath and motion XYZ/E are not activated.

Included source boundaries: AABBTreeIndirect.hpp Tree3f, median-of-three
QuickSelect, indexed face order and left-first strict-less-than nearest hit;
SLA/IndexedMesh.cpp:17-43,139-171 f32 mesh/distance and normal; transformed
raw mesh / ground offset PrintObject.cpp:768-782; sampled contour path and
height rules ContourZ.cpp:33-159; perimeter entity traversal:162-243.

Tree construction/query are serial and independent of available_parallelism,
CPU count, scheduling or floating-point reductions. Node/partition/query
buffers are object-owned; query/sample loops allocate no per-point objects.
The source float precision boundaries and original junction retention remain.
Source Line.hpp:282-295 governs collinear/zero-length segment distance.
Source Linef length uses squared-norm sqrt before ceil(length/0.1).

The existing ExtrusionPath receives source z_contoured defaultfalse
(ExtrusionEntity.hpp:166). Copies in seam/scarf split preserve the original
flag. Contoured path replacement clears fitting as a new source Polyline3
would; simplify bypass is ExtrusionEntity.cpp:33-47. Other constructors only
supply the source default; no behavioral assertion is removed or weakened.
The perimeters-only stage runs after fill/ironing creation and skips the
first physical layer. Fill carrier traversal/emission remains the next phase.

Before production, isolated failure inventory and six analytic tests were
written. Tests prove nearest/backface/absent/nonpositive hit, equal-distance
left precedence, repeatable complete tree topology, original junctions,
far/no-hit inactivity, top/wall height bounds and ironing's separate bound.

Next: dedicated FillExtrusionPath -> Polyline3/fitting mechanical-only commit,
source defaultZ0/flagfalse, owned-buffer/capacity preservation, no per-point
allocation/clone. Verify whole-output behavior in both Native modes. Then
activate fill contour and source GCode XYZ/E/travel branches, with separate
obsolete-capability-test retirement before deleting the guard.
