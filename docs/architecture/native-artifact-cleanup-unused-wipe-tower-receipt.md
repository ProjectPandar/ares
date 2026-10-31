# Native 清理 B：删除未消费的 wipe-tower receipt

owner worker6；基线 `bf0dd12b8`。协调者明确允许本片删除与文档合成一个提交，
无需人为 RED；没有行为变化，也不新增 Rust 路径存在性测试。

## 源码边界与结论

OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`：
`libslic3r/GCode/GCodeProcessor.cpp:3163-3165` 的 `WIPE_TOWER_END` 调用
`UsedFilaments::process_wipe_tower_cache`；`:5935-5948` 的
`GCodeProcessor::process_filaments(ToolChange)` 分别 flush model/support/total。
两类 cache 的 flush 时点不同，因此全局 split 守恒不能推广为逐 extruder id 守恒。

这些语义已有 Rust processor 与 runtime Native oracle 覆盖。本片不新增端口、
processor API、Native build、GDB reader 或缓存框架；既有测试 adapter 不扩展为 pipeline。
共享 oracle/cache 的任何后续整理属于 worker4 的整体 B 范围，本片不吸收。

仅删除 `tests/parity/processor-wipe-tower/native.json`（57 行未使用的 Native 输出），
并将该目录 README 改为 source/runtime contract。删除 README 的历史 Native counts、
hashes 和旧临时目录参照，不把 receipt 改名或替换为常量。所有手写输入、preset、override、
原 Rust 测试、full-stream 比较分支、footer 和五 map 精度断言保持不变。

## 最终消费者审计

对 tracked tree（排除 docs 和上游）以及 scripts/web/crates/tests/.github 分别执行：

```sh
git grep -n -I -E 'processor[-_]wipe[-_]tower' -- . ':(exclude)docs/**' ':(exclude)OrcaSlicer/**'
git grep -n -I -E 'processor[-_]wipe[-_]tower' -- scripts web crates tests .github
```

两项均 **0 matches，exit1**，没有目标 receipt 消费者。
额外搜索 `native.json` 并逐项检查路径：`filament_volume_oracle.rs`、adjacent color
与 processor diagnostic 使用运行时目录；web marker-time 使用另一组显式输入，
不是本组。通用 reader 复核：replay 只消费显式外部目录的 `.3mf` 与对应
`plate_1.gcode`，preset discovery 只遍历 vendor/kind profile JSON，test_script
只扫描 scripts/.cargo/.github 入口；均未形成对此 receipt 的动态消费。
完整命令、exit、命中列表和未改 Rust 文件 SHA 记录在仓库外
`/tmp/worker6-wipe-receipt-audit.json`。

## 已有 runtime 覆盖与验收

以下原测试在 required v2 seeded 环境实际运行，全部 **7/7 PASS**（6.298 秒）：

- `filament_usage_stats::filament_stats_kobra3max_wipe_tower`
- `vg1::vg1_x1c_flush_below_remaining_volume_five_maps_match_native`
- `vg1::vg1_x1c_flush_overflow_five_maps_match_native`
- `vg1::vg1_x1c_repeated_flush_five_maps_match_native`
- `vg1::vg1_x1c_inactive_zero_flush_five_maps_match_native`
- `vg1::vg1_x1c_virtual_and_physical_flush_share_remainder`
- `vg1::vg1_x1c_no_virtual_five_maps_match_native`

既有 `filament_volume_oracle::check` 调用显式 `ARES_PROCGCODE_BIN`，将同一 processed
stream 的 producer label 改为 OrcaSlicer 以接纳 CONFIG_BLOCK，运行
`--process-gcode ... --dump-color-change ...`，按原 extruder slot/id 比较五 map
每一项 f64 bits，并检查全局守恒。Kobra wipe-tower 测试保留其 own emitted footer
和 bucket 非零断言；未声称它已建立 Native 整份 G-code 相等，也不增加 per-id 守恒规则。
运行时同输入审计确认 Native 全局守恒而 per-id 不守恒，证据为
`/tmp/worker6-wipe-receipt-runtime-counterexample-audit.json`；不提交该 Native 输出。

本轮 log/exit 位于 `/tmp/worker6-wipe-receipt-runtime.{log,exit}`；成功 Native 原始
输入/JSON/log 和 Ares statistics 仍只在外部 `filament-volumes-oracle-*` 目录。
Rust、scripts、web、共享缓存和 .config 字节零变化；基线的 workspace/clippy/fmt/wasm
结论不受本片影响。本片没有新 ignore、探针、压制、生产守卫或断言放宽。
