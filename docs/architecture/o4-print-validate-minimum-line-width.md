# O4：Print::validate 的最小线宽 Rust rewrite slice

## 上游边界与诊断

固定源：OrcaSlicer v2.4.2，`8500fcdccaa10b5099ac20d252af3a7c560046f1`。
`src/libslic3r/Print.cpp:1524-1531` 从实际使用的 extruders 取最小/最大 nozzle；`:1543-1556` 的 `validate_extrusion_width` 用 ConfigBase::get_abs_value(key, min_nozzle) 判定：零值 auto 有效，否则 **absolute width <= object layer_height** 拒绝，精确文本 `Too small line width`（不追加 option key）。百分比用 nozzle 基准，不是 layer height；`Config.hpp:1284-1285` 为 FloatOrPercent 绝对值语义。
调用顺序 `Print.cpp:1654-1671`：object `line_width`，有 support/raft 才检 `support_line_width`，随后按 inner-wall、outer-wall、sparse、internal-solid、top、skin、skeleton 顺序遍历 object.all_regions。该分支先于 `:1695-1749` 的 O1 G92/E 校验。`OrcaSlicer.cpp:6063-6094` 将 Print validation 错误发到 stderr，CLI_VALIDATE_ERROR(-51)，Unix exit205；不进入 process/产出 G-code。

## Rust 落点及既有规则

Owner 是 `crates/ares-core/src/print/validate/extrusion_width.rs`，不是 config loading、selector、UI 或 extrusion-flow 生成器。`print/validate.rs::validate_applied_project` 借用已解析的 `BoundedResolvedProjectConfig`（`objects[].object`、`all_regions`、used-filament/nozzle 映射）执行此 Print slice，再执行 O1；validated project/profile composition 不重新加载模型/配置。独立 Print 校验和 validated composition 应同诊断，pure processing 不偷偷增加 Print 校验。平坦 settings/STL 入口以对应 applied object/region config 适配同一谓词，不复制规则。现有轻量 PrintObject/PrintRegion/config views 只是临时兼容 shell，不是自研 Ares pipeline；不得凭配置数组全部 nozzle 替代 upstream 实际使用 extruders。

现有 `options/validation/extrusion_width.rs::validate_extrusion_width_options` 是宽度 > 5×max nozzle 上界；`options/validation/range.rs::validate_line_width_range_options` 是 raw value 的定义域检查。两者均不是 absolute width 与 layer_height 的下界比较；当前 `print/validate.rs` 只拥有 O1，不能把 range/上界规则改名冒充 O4，也不改其 API/错误文本。新增 slice 仅 port 上述 upstream lower-width predicate 与 object/region 调用，不修改 width resolution、auto/default、模板、profiles 或 geometry 算法。

## 包含、延后与退出条件

包含 zero-auto、FloatOrPercent、等号拒绝、对象/region 调用顺序及 support/raft 条件，保持 O2/config loading 在 Print 前、O4 在 O1 前；GUI settings/placed-scene 不门控。不引入默认 profile 豁免或字段修复。上游同 lambda 的 `Too large line width` 上界、layer-height/nozzle 错误、support-tree 检查，以及 `Print.cpp:1673-1689` 独立 bridge-width/薄桥条件虽相邻/部分同文本，均延后；不声称完整 Print::validate parity。

RED 使用矩阵原始导出的 8 个 3MF，字节不变，fresh seeded native `--slice 0`：Creality CR-10 SE、Ender-3、Ender-3 Pro、Ender-3 S1 Plus、Ender-3 V3 KE、Ender-3 V3 SE 0.2 nozzle，LONGER LK10/LK10 Plus (0.2 nozzle)。保留 machine/default process/default filament、input hash、command/status/stdout/stderr、Ares 独立校验及 validated 输出/错误。原生须 exit205、stderr `Too small line width\n`、无 G-code；Ares 成功或其他错误均为真实差异，双方一致不叫 RED。原始 Creality region 的 100% skin/skeleton 等于 0.2 layer，LONGER line_width=0.2 等于 layer；不可通过调宽、降层高或补模板规避。

实现前先保留真实 RED；待 spec 批准才改产品。后续验收需原八例精确拒绝与无 G-code、auto/百分比/等号/合法宽度/对象覆盖的预写正反例，以及生命周期/first-error 对照；工作区 nextest、strict Clippy、fmt、WASM 与当前 base 独立 seeded/official 对照。执行结果属于外部证据/commit message，不放入本决策文档。
