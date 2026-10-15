# Regional Honeycomb ZAA direction inputs

Input-only frozen3MFs; no Native outputs. Approved SPEC072c661ce.
Source: approved Line wide-slab fixture (height2.4mm, sparse40%, classic,
wall0, relative E, no solid ceilings, empty rotation templates, direction45).
Only sparse pattern changes to honeycomb; mesh/profile members preserved.
Shared ZAA min_z0.05 and perimeter-height35; flag-false/flag-true have ZAA
enabled, disabled-zaa-flag-true disables ZAA. Fixed ZIP timestamps.

Upstream: Fill.cpp1235-1242 -> FillBase.cpp311-313 ->
FillHoneycomb.hpp53 / FillHoneycomb.cpp9-80.
Native discovery12/12 accepted, repeat/cross-mode stable, with real
undirected Sparse geometry delta at physical layer2 (object index1).
Ares flag1 matches Native flag0 instead. Both controls pass strict complete
API/CLI output; API and CLI raw bytes equal in every case.

Public tests assert independent Native geometry activity and exact Sparse
XY/E before unchanged strict whole-output checks. Native artifacts stay
in external fresh roots. OrcaRunner::from_env follows the optional-oracle
convention; mandatory gates use ARES_REQUIRE_ORCA=1.

SHA256:
- flag-false.3mf: 27299db2211226b748467cb0978951ce7c50b69065d7ceea5fe020783774b0f8
- flag-true.3mf: 0b62b14e539bda8537af5ba86c0695d99e2720bfad491431dbfd3a6670681c6a
- disabled-zaa-flag-true.3mf: d88cb59ce2a810545e8de2a1e5ba890227305485f4bdd5501a54bda049636f58
