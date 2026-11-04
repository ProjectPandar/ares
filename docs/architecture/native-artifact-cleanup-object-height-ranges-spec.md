# Native 清理：object layer-height ranges 冻结 fallback

本片接管 `runner/object_layer_height_ranges.rs` 及同前缀
adapter；其 complex/range_lookup/region 生产代码与 modifier wall 测试不在本片。
本片只迁移 Native 参照生命周期，不改变 layer range 或 seam 行为。

## 上游边界与 Rust 目标

OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`，路径相对 `src/libslic3r/`。

| 上游源文件 / 函数 | 本片验证边界 |
| --- | --- |
| `Slicing.cpp:234-305` `layer_height_profile_from_ranges` | 固定首层、按范围边界排序/裁剪重叠、EPSILON 窄区间过滤、空隙及顶部默认层高 |
| `Slicing.cpp:807-870` `generate_object_layers` | profile 到完整 object layer schedule 的映射 |
| `PrintObject.cpp:3805-3834` `update_layer_height_profile` | model object ranges 驱动 profile 更新 |
| `SeamPlacer.cpp:1084-1091` | 当前实际 layer height 参与 overhang seam penalty；完整 G-code 覆盖 seam 结果 |
| `PrintApply.cpp:958-1011` `generate_print_object_regions` | 保留原手写 source owners；相邻 region 配置生命周期不在本片实现范围 |

Rust 目标仅既有 CLI `runner/object_layer_height_ranges.rs` 及必要同前缀 helper。
继续调用公开 `ares_core::slice_project_plate_validated`；既有 core layer planning、
region 和 seam 实现不变。测试 adapter 是参照 I/O 的临时兼容壳，不扩展为新 pipeline。
上游 profile/region 新行为、mixed-range 支持、epsilon 或 seam 修复均 deferred。

## 精确范围与合同

保留现有测试名与 active 状态：

- `object_range_layer_height_changes_layer_schedule_and_overhang_seam`：base 一例。
- `object_range_height_boundaries_match_native_full_output`：overlap、gap-top、epsilon 三例。
- `mixed_height_and_region_only_ranges_keep_the_unsupported_boundary`：测试体逐字保留，
  不调用 Native、不加环境 gate，仍精确断言 `UnsupportedProjectFeature("layer_config_ranges")`。

前两项仍逐例断言原完整输入 SHA、Native 全部 `;Z:`/`;HEIGHT:` 序列，保留测试中既有
normative expected layer vectors，不用历史 oracle 输出表替代。Ares 入口参数仍为
原输入、default metadata/overrides、None plate、seed0。仍用原
`compare_ordered_bytes_generator_only` 比较完整有序字节，包括全部 CONFIG_BLOCK、
M73、travel、seam、footer；不增加归一、容差、排序、过滤或只比较部分层。
保留 Ares 错误传播及成功运行的 `input.3mf/native.gcode/ares.gcode/result.json` 外部证据。

五份 3MF 不变；mixed 输入 SHA256 为
`3cfaef750f9eb7c2a1baae910fd919ae63339364f2269907aa5116cd7ace6c1f`。
另四份输入及来源契约见下节。删除四份 `.native-emitted.gcode`（共 3501 行）及对应
四份 `.oracle.json`；不删除其他组产物，不改共享 README、runner/root 注册、cache、
生产 API、.config 或其它 CLI63/retained/channels/marker/KSR 迁移。

## 保留的手写输入/来源契约

以下仅摘录旧 oracle 中手写的来源/输入 SHA、range XML、change、source owners。
不是 Native 运行 receipt；不复制 Native 输出 hashes/bytes/exit、producer 常量或输出 tables。
其余历史 Native 字段随四份 oracle 删除。XML 文本及各输入 SHA 按现有文件保留：

### `object-range-layer-height.3mf`

```json
{
  "source_sha256": "699e9effa47a72c0de5292532aed8b1966813705f55fd0b4b22fdca661b04ee6",
  "input_sha256": "fc938f743bed802de2691134f232015e715d525fc87f9f4b33ce6a51785b9b5f",
  "change": "only appended layer_config_ranges.xml; existing member payloads unchanged",
  "range_xml": "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<objects><object id=\"1\"><range min_z=\"0.3\" max_z=\"0.6\"><option opt_key=\"layer_height\">0.1</option></range></object></objects>\n",
  "source_owners": [
    "Slicing.cpp:234-305,807-870",
    "PrintObject.cpp:3805-3834",
    "SeamPlacer.cpp:1084-1091"
  ]
}
```

### `object-range-layer-height-overlap.3mf`

```json
{
  "source_sha256": "699e9effa47a72c0de5292532aed8b1966813705f55fd0b4b22fdca661b04ee6",
  "input_sha256": "7f078c18fa8eb8119f2215a5c98afbaa629e9946c430259ca962cd1c929bab11",
  "range_xml": "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<objects><object id=\"1\"><range min_z=\"0.4\" max_z=\"0.6\"><option opt_key=\"layer_height\">0.15</option></range><range min_z=\"0.25\" max_z=\"0.48\"><option opt_key=\"layer_height\">0.1</option></range></object></objects>\n",
  "source_owners": [
    "Slicing.cpp:234-305,807-870",
    "PrintApply.cpp:958-1011"
  ]
}
```

### `object-range-layer-height-gap-top.3mf`

```json
{
  "source_sha256": "699e9effa47a72c0de5292532aed8b1966813705f55fd0b4b22fdca661b04ee6",
  "input_sha256": "9a03a0528b31b292a0494c76fdb985c788c91b3928f6176eadf6a85eeefc65c7",
  "range_xml": "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<objects><object id=\"1\"><range min_z=\"0\" max_z=\"0.1\"><option opt_key=\"layer_height\">0.12</option></range><range min_z=\"0.45\" max_z=\"2\"><option opt_key=\"layer_height\">0.15</option></range></object></objects>\n",
  "source_owners": [
    "Slicing.cpp:234-305,807-870",
    "PrintApply.cpp:958-1011"
  ]
}
```

### `object-range-layer-height-epsilon.3mf`

```json
{
  "source_sha256": "699e9effa47a72c0de5292532aed8b1966813705f55fd0b4b22fdca661b04ee6",
  "input_sha256": "ea1ebb3e63343fb0ff3140623a15ea3553995093dbadbec670199778d4ab4099",
  "range_xml": "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<objects><object id=\"1\"><range min_z=\"0.3\" max_z=\"0.30005\"><option opt_key=\"layer_height\">0.12</option></range><range min_z=\"0.4\" max_z=\"0.6\"><option opt_key=\"layer_height\">0.15</option></range></object></objects>\n",
  "source_owners": [
    "Slicing.cpp:234-305,807-870",
    "PrintApply.cpp:958-1011"
  ]
}
```

## Runtime Native 与依赖策略

删除所有 frozen 常量/参数和 optional fallback。每个 Native 比较先调用既有
`OrcaRunner::from_env()`：required 缺依赖必须在读缓存前 FAIL；optional None
通过共享 `tests/support/native_environment` 输出可见 SKIPPED 后立即 return。
不自写 skip，不在缺 Native 时调用 Ares 再伪称 parity，不写 native.gcode/result.json。
纯 mixed rejection case 不经过这个入口。

沿用既有 fixture adapter 模式：原 3MF 字节复制到既有 artifact work，使用 adapter 原 basename `input.3mf`，
构造既有 `ExportedCase` 并调用 `slice_case`。不使用目前绕过命中的 `rerun_reference`，
不修改 cache recipe。producer/Native/launcher/resources/replay/input SHA、lock、
staging、receipt、publish 与完整输出哈希验证都归现有缓存所有；无第二缓存或预热。
成功运行保留 Native/Ares raw、结果、cache trace/receipt；均不提交。

## 真实迁移 RED 与验收

新增同前缀 policy E2E，启动 exact 单个既有 base 测试的子进程，optional 模式、
显式移除 Native 依赖、受控空 artifact root。断言 child 确实执行该测试、成功退出、
可见 SKIPPED，且不写 `native.gcode` 或虚构 `result.json` 比较产物。
当前 fallback 会在 SKIPPED 后继续消费 frozen 并写这些文件，因此产生真实运行时 RED；
不删产物制造编译失败，不改 Native 输出，不把已被片 A 关闭的 required 路径当新 RED。
RED 单独提交/保留日志，再做 GREEN。policy child 不递归运行自身。

GREEN 后四例 required cold/hot 均实际运行，验证原输入 SHA、完整 layer vectors 与整份
字节合同；cache miss→hit 同 producer/request/output SHA，热运行不重切 Native。
warm cache 下 required 缺 Native仍 FAIL；optional exact child 可见 SKIPPED 并无伪参照。
mixed 纯控制在缺 Native 环境仍 active、精确错误不变。

fresh 同轮 v2 seeded/official base→cand，独立 target/cache，逐项列出状态变化，
共同 Ares raw 不变且候选失败集合为基线子集；Native 波动须独立证据并逐例报告，
不放宽严格比较。若相邻生产变更改变本组输出，以新 main 刷新配对和定向。
workspace nextest、strict clippy、fmt、wasm 须全绿。
