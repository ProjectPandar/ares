# Nullable pre-cooling filament variants

Source: beta `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`PrintConfig.hpp:1926,1929`, `PrintConfig.cpp:8879–8886,8908–8916,10154–10159`.
Gather: `PrintConfig.cpp:11862–11882,11971–11977`.
Nullable integer serialization: `Config.hpp:1085–1100`; all-nil footer omission:
`GCode.cpp:8342–8353`.

The unchanged absent-default input is `../variant-retraction/l59.3mf`, SHA256
`908612c123f73537d6cd07b3baaf93e5c48e97f84503030944c330ac3327f23d`.
Neither pre-cooling key is authored there. Registered singleton defaults `{0}`
must gather into two logical entries, not remain singleton registry echoes.

Archives here change only the two arrays inside project settings; every other
ZIP entry is retained. High Flow selects source columns1/3 into logical slots0/1.
The existing nozzle vector `[217,237]`, self indices `[1,2]`, active High Flow and
positive model tool1 activity establish this independent source-column premise.

| Archive | Source temperature / `_nc` | Native gathered result |
| --- | --- | --- |
| `distinct.3mf` | `[121,132,143,154]` / `[161,172,183,194]` | `[132,154]` / `[172,194]` |
| `mixed.3mf` | `[nil,0,123]` / `[0,nil,173]` | `[0,nil]` / `[nil,0]` |
| `all-nil.3mf` | `[nil]` / `[0]` | first key absent; `_nc` `[0,0]` |

The mixed case distinguishes in-range zero/nil and missing-column front values.
Its three-entry source deliberately avoids the singleton/per-filament legacy
load normalization shape (`PrintConfig.cpp:11431–11457`).
The all-nil case checks normal omission and no raw/default resurrection. No
cooldown, fan or ramming algorithm is introduced by the member port.

Require Native first/replay stability, successful public API/CLI, positive
tool1 model activity and raw API/CLI equality. Compare both complete labelled
footer lines, including absence for all-nil. Keep strict complete-output
comparisons separate; no broad whole-output parity claim.

Failure modes: registered defaults not materialized, wrong source index/order,
nil replaced by zero or vice versa, raw vectors leaking into output, duplicate
typed/raw lines, all-nil resurrection, ignored `_nc` distinction, or API/CLI
serialization drift. Tests exercise public loading and slicing, not private
helpers. Fixture SHA256 identities are pinned in the tests.
