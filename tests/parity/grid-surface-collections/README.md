# Grid surface collection fixtures

These are Native-exported input projects, not golden G-code. Upstream revision
`8500fcdccaa10b5099ac20d252af3a7c560046f1`; owning behavior is
`Fill/Fill.cpp:1326-1357` → `Fill/FillBase.cpp:161-182`.

| Input | Purpose | SHA256 |
|---|---|---|
| k1-bridge.3mf | per-surface Grid collection target | 3750e02942d25b6a96866da9990a6c4bd8bc30bfb0eb990ed64c4f109ab9badb |
| mk4-bridge.3mf | passing stock machine control | 999f4fc56915b6591b653bf4e1909c85eef6319da306947a73cb1d89aaeeee3b |

Both use upstream `tests/data/bridge.obj`, SHA256
`379e726104004fc5632d86c02b5242769162dc02057096899ee520f74ef8e8f3`.
VendorProfiles flattening resolved genuine vendor inheritance, without BBL ID
substitution. K1 presets: `Creality K1 (0.4 nozzle)`,
`0.20mm Standard @Creality K1 (0.4 nozzle)`, `Creality HF Generic PLA`.
MK4: `Prusa MK4 0.4 nozzle`, `0.20mm Standard @MK4`,
`Prusa Generic PLA @MK4`. Official OrcaSlicer 2.4.2 exported each project:

```text
orca-slicer --load-settings machine.json;process.json
  --load-filaments filament.json --arrange 1 --export-3mf project.3mf bridge.obj
```

Exact export commands, flattened JSON and Native exit0 records remain external:
`${EVIDENCE_ROOT}/w2-geometry-stock-20261024T224939/{k1,mk4}/bridge/`.
K1 profile JSON is under
`${EVIDENCE_ROOT}/w6-k1-voron-discovery-20261024T223329/flatten-Creality/`;
MK4 under `${EVIDENCE_ROOT}/w6-nonbbl-stock-baseline-ol75m7h7/mk4/`.
Input hashes are asserted by the shared `native_stream_case` harness. Every
formal invocation qualifies stable Native double replay before current public
API/actual CLI and full ordered comparison; generator line only is normalized.
The MK4 control is not claimed to isolate Grid collection granularity.

Read-only live trace proved matching grouped expolygons and all generated
vertices, with Native collection entity counts[1,1] versus Ares[2]. Evidence:
`${EVIDENCE_ROOT}/w2-grid-collection-trace-20261024T231048/`.
No probe or Native output belongs in this fixture directory.
