# C1 PrintObject identity in existing label consumers

C1 scope includes emitted group identity/membership and footprint frame.
GCode.cpp:8084-8118 sets print-object ids and per-group instance/unique ids;
:5348-5352 writes the printing comment and :5360-5372 writes exclusion starts.
These consumers must use the same actual PrintObject/member as the centered
geometry. A source ModelObject index is not an interchangeable print-group index.
No public load/selection contract or new option is added.

In the first attempt, each mode's C1 three strict controls passed and ten
multi-group cases panicked in island_print_order.rs141 before emitted comparison;
R controls 38/38 passed. The source-only infill_first vector must instead be built per actual print group.
Static audit identified source-index use in ObjectLabels/footprint too.

Supplemental input rotation-exclusion copies rotation-group geometry and changes
only exclude_object to1. All original thirteen 3MF SHAs remain unchanged. The
public test asserts that active setting and compares full Native/API/CLI output.
Its Native RED precedes this consumer fix. No existing assertion
or strict comparator changes. Native products remain outside Git.
