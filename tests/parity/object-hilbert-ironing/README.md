# Object Hilbert ironing: Native-active public admission RED

Source8500fcdccaa10b5099ac20d252af3a7c560046f1:
Fill.cpp:1611-1640/1705-1731; FillBase.cpp:62; FillPlanePath.cpp:69-164/248-275.

Frozen input-only10x10x1mm box,8vertices12orientedtriangles,5layers,top_shell_layers1;
no support/raft/tower/brim/skirt. Complete SHA/member manifests in manifest.json.
Target SHAa741b0504891bd65253418a9a59d949c9e0ab450f2be66085a02a96d9ec60a3d.
- hilbert-active: topmost/hilbertcurve.
- rectilinear-control: ONLY ironing_pattern changes to rectilinear.
- hilbert-disabled-control: ONLY ironing_type changes to no ironing.

The target is TOPMOST: all5 MODEL layers
must positively extrude; ONLY last layer has actual ironing, NOT all5 layers.
Native FIRST must slice successfully, preserve configured pattern/type, emit
positive Ironing XY+E only on layer5 for target/patterncontrol, none for disabled.
API/actualCLI must then succeed, rawAPI==CLI, and COMPLETE ordered Native equality
through unchanged golden::compare_ordered_bytes_generator_only.

Baseline target MUST genuinely FAIL at existing UnsupportedProjectFeature
("ironing_pattern") guard, after Native-positive assertions. No success assertion
for rejection; API+CLI errors retained in admission.json. No fabricated target
G-code byte index: API rejects and CLI1 emits no target G-code. Both controls
must pass complete comparison. No Native outputs committed.

GREEN must admit ONLY HilbertCurve in the object
ironing guard and reuse existing main PlanePath/Hilbert implementation as the
single source per Fill.cpp dispatch; no second generator or comparator changes.
