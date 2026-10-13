# Authored layer-range slicing inputs

`generate.py` derives projects from tracked input 3MFs listed by source SHA in
`provenance.json`. The cube is authored as eight vertices/twelve triangles;
model transform and matching source metadata place its bottom at zero.
Range XML and stored profile text are input contracts, not Native captures.

`PrintApply.cpp:342-395,958-1027` owns interval/config partition;
`PrintObjectSlice.cpp:100-136,148-228,244-266` owns volume filtering and routing;
`PrintObject.cpp:3664-3705` owns range-last model-part precedence.
Runtime Native output is generated through existing OrcaRunner caches and kept
outside the repository. Complete-byte comparators are unchanged.

Missing-height without a retained profile is an external diagnostic only;
no acceptance fixture or parity test is registered for that crashing input.
