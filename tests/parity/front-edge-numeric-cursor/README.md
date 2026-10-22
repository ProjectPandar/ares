# Front-edge initial numeric cursor

Input-only Native exports; no expected G-code is committed. Source owner:
`GCode.cpp:5659-5669` unconditional numeric `m_last_pos` translation,
`GCode.hpp:197,224,397-399,607-608` independent defined flag, and
`GCode/SeamPlacer.cpp:1554-1558` nearest preferred point.
Upstream pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
See `docs/architecture/front-edge-initial-numeric-cursor.md`.

Both contain two normal parts, Classic/nearest, no negative volume/scarf/fuzzy/
EFC/skirt/brim/arc fitting. Common GUI scale X100%,Y10%,Z40% and placement produce
world mesh bbox `[25,0.5,0]..[35,1.5,4]`. Only exported setting difference:
`extruder_offset: ["0x0"] -> ["0x2"]`. All other ZIP member contents are identical.
GUI: `PrintConfig.cpp:2305-2313`, `slic3r/GUI/Tab.cpp:5134-5135` Position group.

Frozen copies from accepted discovery evidence:
`/home/indexyz/ares-evidence/w6-92238-front-edge-nearest-br884s6n/{control,target}/exported.3mf`.
`SHA256SUMS.json` binds exact archive bytes, not a reconstructed equivalent.
Discovery control passed official+seeded complete streams; target official first
non-M73 difference at model-locate move `Y-1.25` versus `Y-.75`. These coordinates
are explanatory evidence, NOT expected constants in the tests.

Owner audit root `/home/indexyz/ares-evidence/w6-front-edge-owner-audit-pvpa3_kb/`:
Native preferred `(-30,-1)` in both; Ares None fallback gives target `(-30,+1)`.
Four complete read-only observers exited normally with unchanged outputs; equal
candidate geometry/ranges/angles/overhang/embedded and visibility evidence isolate
the preferred cursor input. HOST replay is not a runtime index/expf capture.

Public tests call the unchanged shared `native_stream_case::check`/`NativeCase`
with `(false, |_, _| {})`: Native2 stable FIRST, actual byte API and real CLI,
whole ordered generator-only comparison. No masks, tolerance or expected rejection.
RED target canonical panic occurs before final raw assertion/adapter update;
retained actual API/CLI raw equality must be checked independently.

This new positive/front-edge witness does not revive discarded cursor commits
489/66/dd8 or use their tests/production changes. No production code is included.
