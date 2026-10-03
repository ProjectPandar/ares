# Support connector arc contract

Port the source FillBase.cpp connector availability, current-endpoint trimming,
equal-X vertical extension and libslic3r.h scaled epsilon into the existing
fill/base_support module (ARD0093). No alternate Ares connector is introduced.

Included: all active availability/trim/mark/cap epsilon consumers and the
already diagnosed vertical endpoint/direction corrections. Deferred: new
edge-input fixture combinations, other Fill algorithms and option sweeps.

Verify a source-closed independent snapshot with existing base-support/Fill
behavior tests, compilation, fmt and clippy; rerun combined public Normal/Tree
complete-output controls before the coupled runtime batch merges. Preserve all
original fixtures and assertions. Document the lack of a threshold-sensitive
RED fixture rather than adding private constant/source-structure assertions.
