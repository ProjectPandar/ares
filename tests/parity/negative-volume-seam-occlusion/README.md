# Negative-volume seam occlusion — input-only public RED

SPEC: docs/architecture/negative-volume-seam-occlusion.md (ab6803ea4), independent
base0266b34949a6f9503002a3420e1c526cb745462b. Upstream2.4.2 source pin
8500fcdccaa10b5099ac20d252af3a7c560046f1.

Owner: SeamPlacer.cpp628-704 MODEL_PART/NEGATIVE occlusion buckets and128-233
negative visibility; TriangleSetSampling.cpp9-70 sampled triangle IDs;
AABBTreeIndirect.hpp755-787 BVH all-hits. No linear triangle scan is an acceptable
implementation. No production change belongs in this RED.

Both3MFs are byte-identical accepted Native exports from the existing
validation/modifier-wall-loops-classic cube geometry/common X1 Carbon0.4 profile.
Normal control vs negative target changes one GUI part type. Sourceobject2/part3
is Nativeexportobject3/part2 in both. Project settings, other part, transforms,
model/build/mesh payloads are unchanged. Input-only archives contain no G-code.
SHA256SUMS.json freezes original bytes, not expected Native output.

Native normal->negative export also omits six derived source provenance keys:
source_file=authored-input.3mf,source_object_id=0,source_volume_id=1,
source_offset_x/y/z=0. This accepted bookkeeping follows bbs_3mf.cpp2214-2220
MODEL_PART source fallback and7862-7869 conditional export. Physical one-field
archive-only difference is NOT claimed. The original semantic audit remains at
/home/indexyz/ares-evidence/w6-3fb-part-negative-region-u3_w7byu/.

Classic/nearest,extruder_offset0x2,no arc/skirt/brim/EFC/scarf/ironing/painting.
The corrected first producer is missing negative occlusion, NOT initial cursor,
negative Boolean region slicing, nearest score coefficients or time estimation.
Abandoned cursor SPEC489/RED66/candidate dd8 are not dependencies or inherited
changes. The read-only corrected audit is w6-candidate-nearest-audit-ioic_az3.

Public tests reuse unchanged native_stream_case::check -> NativeCase: Native2
first, actual validated byte API and real CLI, full generator-only ordered
comparison and actual raw equality. No Native visibility/coordinate/time expected
constants, inverted assertion, Unsupported guard, mask, tolerance or helper edit.
On target RED canonical panic, final raw assertion/adapter update has not run;
independently audit retained actual API/CLI bytes without claiming it executed.

Run only test(negative_volume_seam_occlusion), CPU15/jobs1/threads1, independent
initially empty official/seeded caches. Normal must completePASS, target genuine
RED. No GREEN/FULL/integration, rebase/push/merge before coordinator approval.
