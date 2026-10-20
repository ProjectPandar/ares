# TPMS-D anchoring surface contraction inputs

Input-only3MFs for approved SPEC `4500c1adf`.
Upstream: `Fill.cpp:1477-1497` sparse anchoring calls base
`FillBase.cpp:105-118` contraction before `FillTpmsD.cpp:101-160`.

The authored anchoring mesh/profile bundle is unchanged. These four TPMS-D
inputs differ only in `top_shell_layers` and `fill_multiline`; all other
metadata and ZIP members are identical. Each contains nine input members,
no G-code/preview output, and no stored Native golden. Direction45, sparse20%,
layer0.2,10mm model and both ZAA bits0 are common. Top6/top7/top5 bridge at
physical45/44/46 and project anchoring indices43/42/44, respectively.

| File | Role | SHA256 |
|---|---|---|
| top6-multiline2.3mf | target | 1e250882f206fd72c46c8a89349cd0e6ce79c3cfe392b819c188c0b3a87bc46f |
| top7-multiline2.3mf | target | cb6346529e9aa6455c46b90d33a95b2fa25d05bd09f648b9898615027e29c9c4 |
| top5-multiline2.3mf | phase control | 68062dd99ad802baab49e21b2ab2c0cd8746f576c3e7ae1193671068cf5aa63f |
| top6-multiline1.3mf | count control | 874443522220d36b71b0c717cb8a93498540f0d018da789d1fe7be240f052989 |

The public tests reuse the existing shared anchoring harness without changing
its assertions or observer: pinned SHA, public loaded options, Native footer,
positive bridge activity, unchanged ordinary Sparse, fresh Native repeats,
stable API repeats and raw API/CLI equality. Strict complete-file Native
comparison includes all E, wipe/retraction, feedrate and M73 commands.
Expected RED per mode: both targets FAIL, both controls PASS. Existing
Honeycomb/Line cases remain unchanged passing controls.

Top6/multiline2 first discovered difference is bridge E.05321/E.05322;
top7/multiline2 retains the following-layer wipe E-.3025/E-.30251. No last-
decimal tolerance or formatting fix is accepted. Only after separate RED
approval may GREEN restore the anchoring base contraction; any remaining
strict residual then means STOP/report, not scope expansion. GREEN merge
follows the coordinator's W2-priority queue.
