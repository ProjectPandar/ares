# Per-object skirt / OuterAndInner brim public inputs

Input-only 3MFs from approved discovery outer-inner pair; no Native outputs.
Exact SHA256 manifest. Geometry and all non-config ZIP members are identical;
project settings differ only in skirt_type (target perobject, control combined).
Two separate print objects, one instance each, one filament, ByLayer,
outer_and_inner width8 gap0, combine_brims0, no support/raft/tower.
No hole/inner-hole extrusion claim: actual active outer brim is the witness.
Source Print.cpp:2780-2977 and Brim.cpp:448-460,890,948-977.

Runtime Native FIRST; both cases assert strict whole Native/API/CLI equality
and raw API==CLI. Native must produce two brim blocks and one/two skirt blocks
for combined/perobject respectively, with effective settings and positive XY/E.
The target must fail on current real unsupported API rejection until approved
GREEN. No inverse divergence-as-success assertion or comparator relaxation.
Auto/InnerOnly/combine1/multiple-instance guard scopes are not admitted here.
