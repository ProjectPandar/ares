# Line anchoring regional ZAA inputs

Input-only 3MFs for approved SPEC `aee2c9de1`.
Upstream: `Fill.cpp:1444-1446` printing-region conjunction →
`FillBase.cpp:311-313` / `FillBase.hpp:211` odd-layer direction →
`FillLine.hpp:12-25`, `FillLine.cpp:10-119`.

Use the approved authored Honeycomb anchoring mesh/profile bundle as input
provenance, changing pattern to Line and multiline to1. These five files then
differ only in `zaa_enabled`, `zaa_dont_alternate_fill_direction`, and
`top_shell_layers`. Other metadata and ZIP members are preserved. Each has
nine input members and no G-code/preview output; no Native goldens are stored.

Common settings: direction45, sparse20%, layer height0.2, ZAA min_z0.05 and
perimeter-height35. The 10mm model has50 object layers. Top6 projects anchors
from odd object index43 and bridges at physical45; top5/top7 project even
indices44/42 and bridge at physical46/44. This is Line's modulo-two lifecycle,
not Honeycomb's modulo-three cycle.

| File | Role | SHA256 |
|---|---|---|
| top6-enabled-flag1.3mf | target | ce4695fd7095f07426733d26400c6d93ae8cab0fee0b04fe5fd5bed98d5f310f |
| top6-enabled-flag0.3mf | conjunction control | aea1bfca56737f4b58e2b2321bb90d067b814c9b406960b15ca6475376591472 |
| top6-disabled-flag1.3mf | conjunction control | f16bbca3785434c6e41e390e27ff3f4b87217413f7bfa694aa69fa4f40c7e3b8 |
| top5-enabled-flag1.3mf | even-phase control | fbe15bc57cac6a87b53e2afe4a447420bf991e1f24b87c465eff7e95e7493397 |
| top7-enabled-flag1.3mf | even-phase control | 2211372b011e9f8ff10e8548d52fd56c57144e97479968212c9f7b34461b6756 |

Public E2E structure follows the approved Honeycomb anchoring ZAA controls:
pinned input SHA, loaded regional settings, Native effective footer, positive
Internal Bridge activity, unchanged ordinary Sparse motion, two fresh Native
runs, stable Native/API repeats and public API/CLI raw byte equality. The
unchanged strict whole-file comparator retains all M73 and feedrate commands.

Expected RED: only top6 enabled1/flag1 fails, four controls pass in both modes.
Its discovered first strict delta is byte3099/line143 (`M73 P10 R7`), with
later bridge-entry geometry and same-layer cooling feedrate effects. Neither
M73 nor cooling/feedrate handling is part of the future bounded port. The
extra diagnostic top6 disabled0/flag0 case remains external, not a sixth
public case. Production is unchanged until separate RED approval.
